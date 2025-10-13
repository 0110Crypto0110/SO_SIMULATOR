use std::sync::Arc;
use tokio::sync::{OwnedSemaphorePermit};
use tokio::time::{sleep, Duration};

use crate::recursos::Recursos;
/// Aguarda um recurso específico (médico, sala, leito)
/// e retorna o *permit* que deve ser mantido enquanto o recurso está em uso.

pub async fn esperar_recurso(nome_recurso: &str, paciente: &str, recursos: Arc<Recursos>,) -> OwnedSemaphorePermit {

    // adiciona paciente na fila visível ao monitor
    recursos.adiciona_fila(nome_recurso, paciente);
    println!("🕓 [{}] aguardando recurso: {}", paciente, nome_recurso);

    // tenta adquirir o semáforo correto conforme o tipo de recurso
    let permit = match nome_recurso {
        "Médico" => recursos
            .medicos
            .clone()
            .acquire_owned()
            .await
            .expect("falha ao adquirir semáforo de Médico"),
        "Sala de Cirurgia" => recursos
            .salas_cirurgia
            .clone()
            .acquire_owned()
            .await
            .expect("falha ao adquirir semáforo de Sala de Cirurgia"),
        "Leito" => recursos
            .leitos
            .clone()
            .acquire_owned()
            .await
            .expect("falha ao adquirir semáforo de Leito"),
        other => panic!("Tipo de recurso desconhecido em esperar_recurso(): {}", other),
    };

    // atualiza após obter o recurso
    recursos.remove_fila(nome_recurso, paciente);
    recursos.adiciona_uso(nome_recurso, paciente);

    println!("✅ [{}] obteve {}", paciente, nome_recurso);
    permit
}

/// Simula o uso de um recurso por um determinado tempo
pub async fn usar_recurso(descricao: &str, duracao: u64) {
    println!("🔧 {} em uso por {} segundos...", descricao, duracao);
    sleep(Duration::from_secs(duracao)).await;
    println!("🏁 {} liberado!", descricao);
}

/// Função auxiliar para pausar a execução entre etapas (para logs visíveis)
pub async fn pausa(segundos: u64) {
    sleep(Duration::from_secs(segundos)).await;
}
