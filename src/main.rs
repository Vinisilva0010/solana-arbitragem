use reqwest::Client;
use solana_sdk::{
    signature::{read_keypair_file, Keypair, Signer},
};
use std::{thread, time::Duration};
use std::fs::OpenOptions;
use std::io::Write;
use serde::{Deserialize, Serialize};
use serde_json::Value;

// Estruturas para conversar com a Jupiter
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SwapRequest {
    quote_response: Value,
    user_public_key: String,
    wrap_and_unwrap_sol: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SwapResponse {
    swap_transaction: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::builder().danger_accept_invalid_certs(true).build()?;

    // --- 🔑 SUA CONFIGURAÇÃO ---
    let my_api_key = "f2d3c2aa-92ce-4d98-8795-e5936ae8b597"; 
    let sol_mint = "So11111111111111111111111111111111111111112";
    let usdc_mint = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
    
    // Valor inicial realista para teste (0.05 SOL)
    // É difícil lucrar arbitragem com muito pouco (0.01) por causa das taxas fixas.
    let sol_inicial = 50_000_000u64; 

    // Tenta carregar a carteira (Só para pegar a Public Key, não vamos gastar nada)
    let wallet = match read_keypair_file("meu-bot.json") {
        Ok(w) => w,
        Err(_) => Keypair::new(), // Se não achar, cria uma fake só pra simular
    };

    println!("🛡️ BOT ARBITRAGEM - MODO SEGURO (REALISTA)");
    println!("Carteira Simulada: {}", wallet.pubkey());
    println!("Investimento por operação: {:.4} SOL", sol_inicial as f64 / 1e9);
    println!("Taxa Mínima exigida (Break-even): 80.000 lamports");
    println!("---------------------------------------------------");
    println!("🔎 Escaneando mercado... (Pode demorar para achar algo real)");

    loop {
        // 1. Cotação IDA: SOL -> USDC
        let url_ida = format!(
            "https://api.jup.ag/swap/v1/quote?inputMint={}&outputMint={}&amount={}&slippageBps=50",
            sol_mint, usdc_mint, sol_inicial
        );

        // Faz a requisição...
        if let Ok(resp_ida) = client.get(&url_ida).header("x-api-key", my_api_key).send().await {
            if let Ok(json_ida) = resp_ida.json::<Value>().await {
                
                // Pega quanto USDC receberíamos
                if let Some(usdc_out_str) = json_ida.get("outAmount").and_then(|v| v.as_str()) {
                    let usdc_recebido: u64 = usdc_out_str.parse().unwrap_or(0);

                    // 2. Cotação VOLTA: USDC -> SOL
                    let url_volta = format!(
                        "https://api.jup.ag/swap/v1/quote?inputMint={}&outputMint={}&amount={}&slippageBps=50",
                        usdc_mint, sol_mint, usdc_recebido
                    );

                    if let Ok(resp_volta) = client.get(&url_volta).header("x-api-key", my_api_key).send().await {
                        if let Ok(json_volta) = resp_volta.json::<Value>().await {
                            
                            // Pega quanto SOL volta pra gente
                            if let Some(sol_final_str) = json_volta.get("outAmount").and_then(|v| v.as_str()) {
                                let sol_final: u64 = sol_final_str.parse().unwrap_or(0);

                                // --- 🧮 A MATEMÁTICA DA VERDADE ---
                                let diff = sol_final as i64 - sol_inicial as i64;
                                
                                // CUSTO ESTIMADO DA REDE (Fees + Priority)
                                // 5.000 (Base) + 75.000 (Priority pra garantir)
                                let custo_estimado_rede = 80_000; 

                                if diff > custo_estimado_rede {
                                    let lucro_liquido = diff - custo_estimado_rede;
                                    
                                    // TRAVA DE SEGURANÇA CONTRA BUGS (Honeypot/Erro de API)
                                    // Se o lucro for maior que 50% do investimento em segundos, é mentira.
                                    let lucro_maximo_possivel = sol_inicial as i64 / 2; // 50%

                                    if lucro_liquido > lucro_maximo_possivel {
                                        println!("\n⚠️ ALERTA DE BUG/HONEYPOT NA API!");
                                        println!("Lucro prometido absurdo: {} lamports. Ignorando.", lucro_liquido);
                                    } else {
                                        // AQUI SIM É UM LUCRO REAL
                                        println!("\n✅ OPORTUNIDADE REAL ENCONTRADA!");
                                        println!("Entrada: {:.4} SOL | Saída: {:.4} SOL", sol_inicial as f64/1e9, sol_final as f64/1e9);
                                        println!("Lucro Bruto: {} lamports", diff);
                                        println!("Lucro Líquido (Desc. Taxas): +{} lamports", lucro_liquido);
                                        
                                        // Salva no log
                                        if let Ok(mut arquivo) = OpenOptions::new().create(true).append(true).open("log_seguro.txt") {
                                            let linha = format!("REAL: Lucro Líquido {} lamports | Data: {:?}\n", lucro_liquido, std::time::SystemTime::now());
                                            let _ = arquivo.write_all(linha.as_bytes());
                                        }

                                        // Dorme mais tempo se achar oportunidade para não floodar o log com a mesma coisa
                                        thread::sleep(Duration::from_secs(10)); 
                                    }
                                } else {
                                    // Só imprime ponto para saber que está rodando
                                    print!("."); 
                                    let _ = std::io::stdout().flush();
                                }
                            }
                        }
                    }
                }
            }
        }
        // Intervalo entre checagens (evita bloqueio da API)
        thread::sleep(Duration::from_secs(2));
    }
}