use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Instant, Duration};

pub struct Estatisticas {
    // Registra o tempo total de atendimento por paciente (String)
    pub(crate) atendimentos: Arc<Mutex<HashMap<String, Duration>>>,
}

impl Estatisticas {
    /// Retorna uma referência clonada ao Arc<Mutex> para acesso da GUI
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
    /// O parâmetro 'concluido' é crucial para ignorar pacientes abortados pelo monitor.
    pub fn finalizar_atendimento(&self, paciente: &str, inicio: Instant, concluido: bool) {
        let duracao = inicio.elapsed();
        
        if concluido {
            println!("✅ Fim do atendimento: {} ({:.2}s)", paciente, duracao.as_secs_f64());
            let mut lock = self.atendimentos.lock().unwrap();
            lock.insert(paciente.to_string(), duracao);
        } else {
            // Log de um atendimento cancelado/abortado, mas não adiciona às estatísticas
            println!("❌ Atendimento CANCELADO/ABORTADO: {} ({:.2}s)", paciente, duracao.as_secs_f64());
        }
    }

    /// Imprime relatório consolidado de atendimentos CONCLUÍDOS
    pub fn imprimir_relatorio(&self) {
        let lock = self.atendimentos.lock().unwrap();
        println!("\n📊 Relatório Final de Atendimentos CONCLUÍDOS:");
        
        let mut total_atendidos = 0;
        let mut total_tempo: f64 = 0.0;
        
        for (paciente, duracao) in lock.iter() {
            println!(" - {} → {:.2} segundos", paciente, duracao.as_secs_f64());
            total_tempo += duracao.as_secs_f64();
            total_atendidos += 1;
        }

        let media: f64 = if total_atendidos > 0 {
            total_tempo / total_atendidos as f64
        } else { 0.0 };

        println!("\nTotal de atendimentos CONCLUÍDOS: {}", total_atendidos);
        println!("Tempo total acumulado: {:.2} segundos", total_tempo);
        println!("Tempo médio por paciente: {:.2} segundos", media);
    }
}