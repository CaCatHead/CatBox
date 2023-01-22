use super::context::CatBoxParams;

pub fn make_compile_params(language: String, _submission: String) -> CatBoxParams {
  match language.as_str() {
    "c" | "cpp" | "c++" => {
      let args = vec![];
      let params = CatBoxParams::new("g++".to_string(), args);
      params
    }
    _ => {
      unimplemented!()
    }
  }
}
