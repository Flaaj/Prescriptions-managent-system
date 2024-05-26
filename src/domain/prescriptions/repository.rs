use std::sync::RwLock;

use crate::domain::{
    prescriptions::models::{NewPrescription, NewPrescriptionFill, Prescription, PrescriptionFill},
    utils::pagination::get_pagination_params,
};
use async_trait::async_trait;
use chrono::Utc;
use uuid::Uuid;

use super::models::{PrescribedDrug, PrescriptionDoctor, PrescriptionPatient};

#[async_trait]
pub trait PrescriptionsRepository {
    async fn create_prescription(
        &self,
        prescription: NewPrescription,
    ) -> anyhow::Result<Prescription>;
    async fn get_prescriptions(
        &self,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> anyhow::Result<Vec<Prescription>>;
    async fn get_prescription_by_id(&self, prescription_id: Uuid) -> anyhow::Result<Prescription>;
    async fn fill_prescription(
        &self,
        prescription_fill: NewPrescriptionFill,
    ) -> anyhow::Result<PrescriptionFill>;
    // async fn get_prescriptions_by_prescription_id(&self, prescription_id: Uuid) -> anyhow::Result<Vec<Prescription>>;
    // async fn get_prescriptions_by_patient_id(&self, patient_id: Uuid) -> anyhow::Result<Vec<Prescription>>;
    // async fn update_prescription(&self, prescription: Prescription) -> anyhow::Result<()>;
    // async fn delete_prescription(&self, prescription_id: Uuid) -> anyhow::Result<()>;
}

/// Used to test the service layer in isolation
pub struct InMemoryPrescriptionsRepository {
    prescriptions: RwLock<Vec<Prescription>>,
}

impl InMemoryPrescriptionsRepository {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            prescriptions: RwLock::new(Vec::new()),
        }
    }
}

#[async_trait]
impl PrescriptionsRepository for InMemoryPrescriptionsRepository {
    async fn create_prescription(
        &self,
        new_prescription: NewPrescription,
    ) -> anyhow::Result<Prescription> {
        let prescription = Prescription {
            id: new_prescription.id,
            doctor: PrescriptionDoctor {
                id: new_prescription.doctor_id,
                name: "John Doe".into(),
                pesel_number: "12345678900".into(),
                pwz_number: "1234567".into(),
            },
            patient: PrescriptionPatient {
                id: new_prescription.patient_id,
                name: "John Doe".into(),
                pesel_number: "12345678900".into(),
            },
            prescribed_drugs: new_prescription
                .prescribed_drugs
                .iter()
                .map(|new_prescibed_drug| PrescribedDrug {
                    id: Uuid::new_v4(),
                    drug_id: new_prescibed_drug.drug_id,
                    prescription_id: new_prescription.id,
                    quantity: new_prescibed_drug.quantity as i32,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                })
                .collect(),
            prescription_type: new_prescription.prescription_type,
            code: new_prescription.code,
            fill: None,
            start_date: new_prescription.start_date,
            end_date: new_prescription.end_date,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        self.prescriptions
            .write()
            .unwrap()
            .push(prescription.clone());

        Ok(prescription)
    }

    async fn get_prescriptions(
        &self,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> anyhow::Result<Vec<Prescription>> {
        let (page_size, offset) = get_pagination_params(page, page_size)?;
        let a = offset;
        let b = offset + page_size;

        let mut prescriptions: Vec<Prescription> = vec![];
        for i in a..b {
            match self.prescriptions.read().unwrap().get(i as usize) {
                Some(prescription) => prescriptions.push(prescription.clone()),
                None => {}
            }
        }

        Ok(prescriptions)
    }

    async fn get_prescription_by_id(&self, prescription_id: Uuid) -> anyhow::Result<Prescription> {
        match self
            .prescriptions
            .read()
            .unwrap()
            .iter()
            .find(|prescription| prescription.id == prescription_id)
        {
            Some(prescription) => Ok(prescription.clone()),
            None => Err(anyhow::anyhow!("Prescription not found")),
        }
    }

    async fn fill_prescription(
        &self,
        new_prescription_fill: NewPrescriptionFill,
    ) -> anyhow::Result<PrescriptionFill> {
        let prescription_fill = PrescriptionFill {
            id: new_prescription_fill.id,
            prescription_id: new_prescription_fill.prescription_id,
            pharmacist_id: new_prescription_fill.pharmacist_id,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let prescriptions = self.prescriptions.read().unwrap().to_owned();
        let (index, prescription) = prescriptions
            .iter()
            .enumerate()
            .map(|(index, prescription)| {
                (index, {
                    let mut prescription = prescription.clone();
                    prescription.fill = Some(prescription_fill.clone());
                    prescription
                })
            })
            .find(|(_, prescription)| prescription.id == new_prescription_fill.prescription_id)
            .unwrap();

        self.prescriptions
            .write()
            .unwrap()
            .insert(index, prescription);

        Ok(prescription_fill)
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use crate::domain::{
        doctors::{
            models::NewDoctor,
            repository::{DoctorsRepository, InMemoryDoctorsRepository},
        },
        drugs::{
            models::{DrugContentType, NewDrug},
            repository::{DrugsRepository, InMemoryDrugsRepository},
        },
        patients::{
            models::NewPatient,
            repository::{InMemoryPatientsRepository, PatientsRepository},
        },
        pharmacists::{
            models::NewPharmacist,
            repository::{InMemoryPharmacistsRepository, PharmacistsRepository},
        },
        prescriptions::{
            models::NewPrescription,
            repository::{InMemoryPrescriptionsRepository, PrescriptionsRepository},
        },
    };

    struct DatabaseSeedData {
        doctor: NewDoctor,
        pharmacist: NewPharmacist,
        patient: NewPatient,
        drugs: Vec<NewDrug>,
    }

    async fn seed_database() -> anyhow::Result<DatabaseSeedData> {
        let pharmacists_repo = InMemoryPharmacistsRepository::new();
        let pharmacist = NewPharmacist::new(
            "John Pharmacist".into(), //
            "96021807250".into(),
        )?;
        pharmacists_repo
            .create_pharmacist(pharmacist.clone())
            .await?;

        let patients_repo = InMemoryPatientsRepository::new();
        let patient = NewPatient::new(
            "John Patient".into(), //
            "96021807250".into(),
        )?;
        patients_repo.create_patient(patient.clone()).await?;
        let drugs_repo = InMemoryDrugsRepository::new();
        let mut drugs = vec![];
        for _ in 0..4 {
            let drug = NewDrug::new(
                "Gripex".into(),
                DrugContentType::SolidPills,
                Some(20),
                Some(300),
                None,
                None,
            )?;
            drugs.push(drug.clone());
            drugs_repo.create_drug(drug).await?;
        }

        let doctors_repo = InMemoryDoctorsRepository::new();
        let doctor = NewDoctor::new(
            "John Doctor".into(), //
            "3123456".into(),
            "96021807250".into(),
        )?;
        doctors_repo.create_doctor(doctor.clone()).await?;

        Ok(DatabaseSeedData {
            doctor,
            pharmacist,
            patient,
            drugs,
        })
    }

    async fn setup_repository() -> (InMemoryPrescriptionsRepository, DatabaseSeedData) {
        let seed_data = seed_database().await.unwrap();
        let repository = InMemoryPrescriptionsRepository::new();
        (repository, seed_data)
    }

    #[tokio::test]
    async fn creates_and_reads_prescriptions_from_database() {
        let (repository, seed_data) = setup_repository().await;

        let mut new_prescription =
            NewPrescription::new(seed_data.doctor.id, seed_data.patient.id, None, None);
        for i in 0..4 {
            new_prescription.add_drug(seed_data.drugs[i].id, 1).unwrap();
        }

        repository
            .create_prescription(new_prescription.clone())
            .await
            .unwrap();

        for _ in 0..10 {
            let mut another_prescription =
                NewPrescription::new(seed_data.doctor.id, seed_data.patient.id, None, None);
            another_prescription
                .add_drug(seed_data.drugs[0].id, 1)
                .unwrap();
            repository
                .create_prescription(another_prescription)
                .await
                .unwrap();
        }

        let prescriptions = repository.get_prescriptions(None, Some(7)).await.unwrap();

        assert_eq!(prescriptions.len(), 7);
        assert_eq!(prescriptions[0], new_prescription);

        let prescriptions = repository.get_prescriptions(None, Some(20)).await.unwrap();
        assert!(prescriptions.len() == 11);

        let prescriptions = repository
            .get_prescriptions(Some(1), Some(10))
            .await
            .unwrap();
        assert!(prescriptions.len() == 1);
    }

    #[tokio::test]
    async fn creates_and_reads_prescription_by_id() {
        let (repository, seed_data) = setup_repository().await;

        let mut new_prescription =
            NewPrescription::new(seed_data.doctor.id, seed_data.patient.id, None, None);
        for i in 0..2 {
            new_prescription.add_drug(seed_data.drugs[i].id, 1).unwrap();
        }

        repository
            .create_prescription(new_prescription.clone())
            .await
            .unwrap();

        let prescription_from_db = repository
            .get_prescription_by_id(new_prescription.id)
            .await
            .unwrap();

        assert_eq!(prescription_from_db, new_prescription);
    }

    #[tokio::test]
    async fn returns_error_if_prescription_doesnt_exist() {
        let (repository, _) = setup_repository().await;
        let prescription_id = Uuid::new_v4();

        let prescription_from_db = repository.get_prescription_by_id(prescription_id).await;

        assert!(prescription_from_db.is_err());
    }

    #[tokio::test]
    async fn fills_prescription_and_saves_to_database() {
        let (repository, seed_data) = setup_repository().await;

        let mut prescription =
            NewPrescription::new(seed_data.doctor.id, seed_data.patient.id, None, None);
        prescription.add_drug(seed_data.drugs[0].id, 1).unwrap();

        repository
            .create_prescription(prescription.clone())
            .await
            .unwrap();

        let prescription_from_db = repository
            .get_prescription_by_id(prescription.id)
            .await
            .unwrap();

        assert!(prescription_from_db.fill.is_none());

        let new_prescription_fill = prescription_from_db.fill(seed_data.pharmacist.id).unwrap();
        let created_prescription_fill = repository
            .fill_prescription(new_prescription_fill.clone())
            .await
            .unwrap();

        assert_eq!(created_prescription_fill, new_prescription_fill);

        let prescription_from_db = repository
            .get_prescription_by_id(prescription.id)
            .await
            .unwrap();

        assert_eq!(prescription_from_db.fill.unwrap(), new_prescription_fill);
    }
}
