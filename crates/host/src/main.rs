//! prysmview-svc - Binário executável do serviço Windows LocalSystem do PrysmView.

use std::env;

#[cfg(windows)]
use prysmview_host::service::{run_service, run_standalone};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args
        .iter()
        .any(|arg| arg == "--run" || arg == "--console" || arg == "-c")
    {
        println!("Iniciando PrysmView Host Service em modo console/standalone...");
        #[cfg(windows)]
        {
            run_standalone()?;
        }
        #[cfg(not(windows))]
        {
            println!("Host service só pode ser executado em modo console no Windows.");
        }
        return Ok(());
    }

    #[cfg(windows)]
    {
        run_service()?;
    }
    #[cfg(not(windows))]
    {
        println!("Serviço Windows prysmview-svc requer ambiente Windows.");
    }

    Ok(())
}
