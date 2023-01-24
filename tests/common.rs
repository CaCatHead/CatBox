use std::sync::Once;

use flexi_logger::Logger;

static INIT: Once = Once::new();

pub fn setup() {
  INIT.call_once(|| {
    Logger::try_with_str("catj=debug,info")
      .unwrap()
      .start()
      .unwrap();
  });
}
