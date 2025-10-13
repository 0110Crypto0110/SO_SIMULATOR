use std::sync::Arc;
use tokio::time::{sleep, Duration};
use crate::recursos::Recursos;

pub async fn iniciar_monitor(recursos: Arc<Recursos>) {
    loop {
        println!("\n🩺 ===== MONITORAMENTO =====");
        println!("👨‍⚕️ Médicos disponíveis: {}", recursos.medicos.available_permits());
        println!("🏥 Salas disponíveis: {}", recursos.salas_cirurgia.available_permits());
        println!("🛏️ Leitos disponíveis: {}", recursos.leitos.available_permits());

        let med_uso = recursos.uso_medicos.lock().unwrap().clone();
        let med_fila = recursos.fila_medicos.lock().unwrap().clone();
        println!("  Médicos em uso: {:?}", med_uso);
        println!("  Fila por médico: {:?}", med_fila);

        let sal_uso = recursos.uso_salas.lock().unwrap().clone();
        let sal_fila = recursos.fila_salas.lock().unwrap().clone();
        println!("  Salas em uso: {:?}", sal_uso);
        println!("  Fila por sala: {:?}", sal_fila);

        let lei_uso = recursos.uso_leitos.lock().unwrap().clone();
        let lei_fila = recursos.fila_leitos.lock().unwrap().clone();
        println!("  Leitos em uso: {:?}", lei_uso);
        println!("  Fila por leito: {:?}", lei_fila);
        println!("============================\n");

        sleep(Duration::from_secs(3)).await;
    }
}
