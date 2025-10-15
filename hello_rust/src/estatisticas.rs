use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Instant, Duration};

pub struct Estatisticas {
    pub(crate) atendimentos: Arc<Mutex<HashMap<String, Duration>>>,
}

impl Estatisticas {
    pub fn obter_atendimentos(&self) -> Arc<Mutex<HashMap<String, Duration>>> {
        Arc::clone(&self.atendimentos)
    }
    
    pub fn novo() -> Self {
        Self {
            atendimentos: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Marca o início do atendimento e retorna o instante inicial
    pub fn iniciar_atendimento(&self, paciente: &str) -> Instant {
        println!("⏱️ Início do atendimento: {}", paciente);
        Instant::now()
    }

    /// Registra o término do atendimento, calculando a duração
    pub fn finalizar_atendimento(&self, paciente: &str, inicio: Instant) {
        let duracao = inicio.elapsed();
        println!("⏱️ Fim do atendimento: {} ({}s)", paciente, duracao.as_secs());
        let mut lock = self.atendimentos.lock().unwrap();
        lock.insert(paciente.to_string(), duracao);
    }

    /// Imprime relatório consolidado de atendimentos
    pub fn imprimir_relatorio(&self) {
        let lock = self.atendimentos.lock().unwrap();
        println!("\n📊 Relatório Final de Atendimentos:");
        for (paciente, duracao) in lock.iter() {
            println!(" - {} → {} segundos", paciente, duracao.as_secs());
        }

        let total: u64 = lock.values().map(|d| d.as_secs()).sum();
        let media: f64 = if lock.len() > 0 {
            total as f64 / lock.len() as f64
        } else { 0.0 };

        println!("Total de atendimentos: {}", lock.len());
        println!("Tempo total: {} segundos", total);
        println!("Tempo médio: {:.2} segundos", media);
    }
}
