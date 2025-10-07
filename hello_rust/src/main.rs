mod paciente;
mod recursos;
mod sincronizacao;
mod monitor;

use paciente::Paciente;
use recursos::Recursos;
use std::sync::Arc;
use tokio::task;

#[tokio::main]
async fn main() {
    println!("\n🏥 Iniciando Simulador de Hospital — Projeto de SO\n");

    // Inicializa os recursos do hospital
    let recursos = Arc::new(Recursos::novo(5, 2, 10));

    // Clones para o monitor
    let recursos_monitor = recursos.clone();

    // Inicia o monitoramento em uma task paralela
    task::spawn({
        let recursos_clone = recursos_monitor.clone();
        async move {
            monitor::iniciar_monitor(
                recursos_clone.medicos.clone(),
                recursos_clone.salas_cirurgia.clone(),
                recursos_clone.leitos.clone(),
            )
            .await;
        }
    });

    // Cria os pacientes com diferentes condições
    let pacientes = vec![
        Paciente::novo("João", 45, "Dor no peito", true),
        Paciente::novo("Maria", 32, "Fratura no braço", false),
        Paciente::novo("Carlos", 60, "Apendicite", true),
        Paciente::novo("Ana", 25, "Gripe forte", false),
        Paciente::novo("José", 70, "Problemas cardíacos", true),
    ];

    // Lança cada paciente em uma task independente
    let mut tarefas = vec![];
    for paciente in pacientes {
        let recursos_clone = recursos.clone();
        let tarefa = task::spawn(async move {
            paciente.atender(recursos_clone).await;
        });
        tarefas.push(tarefa);
    }

    // Aguarda o término de todos os pacientes
    for tarefa in tarefas {
        let _ = tarefa.await;
    }

    println!("\n✅ Todos os pacientes foram atendidos com sucesso!");
}
