// use llm::{ModelArchitecture, VocabularySource};
// use std::convert::Infallible;
// use std::io::Write;
//
// fn main() {
//     // load a GGML model from disk
//     let llama = llm::load_dynamic(
//         ModelArchitecture::Llama,
//         std::path::Path::new("/Users/frederik/Downloads/gpt4-x-vicuna-13B.ggmlv3.q4_1.bin"),
//         VocabularySource::Model,
//         llm::ModelParameters::default(),
//         llm::load_progress_callback_stdout,
//     )
//     .unwrap_or_else(|err| panic!("Failed to load model: {err}"));
//
//     // use the model to generate text from a prompt
//     let prompt = "A chat between Human and Assistant.\n### Assistant: Hello I am a profressional translator which translates sentences from mangas \n\
//         ### Human: Please translate the following sentences for me in the same structure as they are given to japanese. Each translation is numbered with the same number and in a new line.\n\n1. This is a example sentance\n2. Another sentence to translate.\n###Assistant: 1.";
//     let mut session = llama.start_session(Default::default());
//     let mut buf = String::new();
//     let res = session.infer(
//         llama.as_ref(),
//         &mut rand::thread_rng(),
//         &llm::InferenceRequest {
//             prompt: prompt.into(),
//             parameters: &llm::InferenceParameters::default(),
//             play_back_previous_tokens: false,
//             maximum_token_count: None,
//         },
//         &mut Default::default(),
//         inference_callback(String::from("###Human"), &mut buf),
//     );
//
//     println!("res: {}", buf);
//     match res {
//         Ok(result) => println!("\n\nInference stats:\n{result}"),
//         Err(err) => println!("\n{err}"),
//     }
// }
//
// fn inference_callback(
//     stop_sequence: String,
//     buf: &mut String,
// ) -> impl FnMut(llm::InferenceResponse) -> Result<llm::InferenceFeedback, Infallible> + '_ {
//     move |resp| match resp {
//         llm::InferenceResponse::InferredToken(t) => {
//             let mut reverse_buf = buf.clone();
//             reverse_buf.push_str(t.as_str());
//             if stop_sequence.as_str().eq(reverse_buf.as_str()) {
//                 buf.clear();
//                 return Ok(llm::InferenceFeedback::Halt);
//             } else if stop_sequence.as_str().starts_with(reverse_buf.as_str()) {
//                 buf.push_str(t.as_str());
//                 return Ok(llm::InferenceFeedback::Continue);
//             }
//
//             if buf.is_empty() {
//                 print_token(t)
//             } else {
//                 print_token(reverse_buf)
//             }
//         }
//         llm::InferenceResponse::EotToken => Ok(llm::InferenceFeedback::Halt),
//         _ => Ok(llm::InferenceFeedback::Continue),
//     }
// }
//
// fn print_token(t: String) -> Result<llm::InferenceFeedback, Infallible> {
//     print!("{t}");
//     std::io::stdout().flush().unwrap();
//
//     Ok(llm::InferenceFeedback::Continue)
// }

fn main() {}
