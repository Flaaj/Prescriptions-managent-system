use uuid::Uuid;

use super::create_prescription::NewPrescription;

impl NewPrescription {
    pub async fn commit_to_repository(self, pool: &sqlx::PgPool) -> anyhow::Result<()> {
        self.validate()?;

        let transaction = pool.begin().await?;

        sqlx::query!(
            r#"INSERT INTO prescriptions (id, patient_id, doctor_id, prescription_type, start_date, end_date) VALUES ($1, $2, $3, $4, $5, $6)"#,
            self.id,
            self.patient_id,
            self.doctor_id,
            self.prescription_type as _,
            self.start_date,
            self.end_date
        )
        .execute(pool)
        .await?;

        for prescribed_drug in &self.prescribed_drugs {
            sqlx::query!(
                r#"INSERT INTO prescribed_drugs (id, prescription_id, drug_id, quantity) VALUES ($1, $2, $3, $4)"#,
                Uuid::new_v4(),
                self.id,
                prescribed_drug.drug_id,
                prescribed_drug.quantity as i32
            )
            .execute(pool)
            .await?;
        }

        transaction.commit().await?;

        Ok(())
    }
}
