use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio::time::{sleep, Duration};

/// Exibe periodicamente o status atual dos recursos disponíveis.
pub async fn iniciar_monitor(
    medicos: Arc<Semaphore>,
    salas: Arc<Semaphore>,
    leitos: Arc<Semaphore>,
) {
    loop {
        println!("\n🩺 ===== MONITORAMENTO DO HOSPITAL =====");
        println!("👨‍⚕️ Médicos disponíveis: {}", medicos.available_permits());
        println!("🏥 Salas de Cirurgia disponíveis: {}", salas.available_permits());
        println!("🛏️ Leitos disponíveis: {}", leitos.available_permits());
        println!("=========================================\n");
        sleep(Duration::from_secs(4)).await;
    }
}
