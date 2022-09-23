use anyhow::Result;
use clap::ArgMatches;
use log::*;
use paste::paste;
use std::env;

#[derive(Debug)]
struct PrivConfig {
    web: String,
    src_addr: String,
    cert: String,
    key: String,
    dst_addr: String,
}

static mut CONFIG: Option<&PrivConfig> = None;

fn init_log(m: &ArgMatches) {
    let log_level = {
        let debug = m.is_present("debug");
        let log_level = m
            .value_of("log_level")
            .unwrap_or(if debug { "debug" } else { "warn" });
        match log_level {
            level @ "error"
            | level @ "warn"
            | level @ "info"
            | level @ "debug"
            | level @ "trace" => level,
            _ => "error",
        }
    };
    env::set_var("RUST_LOG", log_level);
    debug!("Log level :{}", log_level);
    pretty_env_logger::init();
}

pub fn init_from_matches(m: &ArgMatches) -> Result<()> {
    unsafe {
        if CONFIG.is_some() {
            panic!("Cannot init twice");
        }
    }

    init_log(m);

    let cert = m.value_of("cert").unwrap_or("").to_string();
    let key = m.value_of("key").unwrap_or("").to_string();
    let web = m.value_of("web").unwrap_or("").to_string();
    let src_addr = m.value_of("src_addr").unwrap().to_string();
    let dst_addr = m.value_of("dst_addr").unwrap().to_string();

    let src_addr = if src_addr.contains(':') {
        src_addr
    } else {
        format!("0.0.0.0:{}", src_addr)
    };

    let priv_config = Box::new(PrivConfig {
        cert,
        web,
        key,
        src_addr,
        dst_addr,
    });

    info!("Config {:#?}", priv_config);

    unsafe {
        CONFIG = Some(Box::leak(priv_config));
    }
    Ok(())
}

macro_rules! impl_getter {
    (_ String, $field:ident) => {
        unsafe { CONFIG.unwrap().$field.clone() }
    };
    ($ret:ty, $field:ident) => {
        paste! {
            pub fn [<get_ $field>]() -> $ret {
                impl_getter!(_ $ret, $field)
            }
        }
    };
}

impl_getter!(String, key);
impl_getter!(String, cert);
impl_getter!(String, src_addr);
impl_getter!(String, dst_addr);
impl_getter!(String, web);
