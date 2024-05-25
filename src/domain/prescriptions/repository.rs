use crate::domain::prescriptions::models::{
    NewPrescription, NewPrescriptionFill, Prescription, PrescriptionFill,
};
use async_trait::async_trait;
use uuid::Uuid;

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
    // async fn get_prescriptions_by_doctor_id(&self, doctor_id: Uuid) -> anyhow::Result<Vec<Prescription>>;
    // async fn get_prescriptions_by_patient_id(&self, patient_id: Uuid) -> anyhow::Result<Vec<Prescription>>;
    // async fn update_prescription(&self, prescription: Prescription) -> anyhow::Result<()>;
    // async fn delete_prescription(&self, prescription_id: Uuid) -> anyhow::Result<()>;
}