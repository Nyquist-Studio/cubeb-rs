// Copyright © 2018 Mozilla Foundation
//
// This program is made available under an ISC-style license.  See the
// accompanying file LICENSE for details.

#[cfg(not(feature = "gecko-in-tree"))]
extern crate cmake;
#[cfg(not(feature = "gecko-in-tree"))]
extern crate pkg_config;

use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

macro_rules! t {
    ($e:expr) => {
        match $e {
            Ok(e) => e,
            Err(e) => panic!("{} failed with {}", stringify!($e), e),
        }
    };
}

#[cfg(feature = "gecko-in-tree")]
fn main() {}

#[cfg(not(feature = "gecko-in-tree"))]
fn main() {
    if env::var("LIBCUBEB_SYS_USE_PKG_CONFIG").is_ok()
        && pkg_config::find_library("libcubeb").is_ok()
    {
        return;
    }

    if !Path::new("libcubeb/.git").exists() {
        let _ = Command::new("git")
            .args(&["submodule", "update", "--init", "--recursive"])
            .status();
    }

    let target = env::var("TARGET").unwrap();
    let windows = target.contains("windows");
    let darwin = target.contains("darwin");
    let freebsd = target.contains("freebsd");
    let android = target.contains("android");
    let mut cfg = cmake::Config::new("libcubeb");

    if darwin {
        let cmake_osx_arch = if target.contains("aarch64") {
            // Apple Silicon
            "arm64"
        } else {
            // Assuming Intel (x86_64)
            "x86_64"
        };
        cfg.define("CMAKE_OSX_ARCHITECTURES", cmake_osx_arch);
    }

    let _ = fs::remove_dir_all(env::var("OUT_DIR").unwrap());
    t!(fs::create_dir_all(env::var("OUT_DIR").unwrap()));

    env::remove_var("DESTDIR");
    let dst = cfg
        .define("BUILD_SHARED_LIBS", "OFF")
        .define("BUILD_TESTS", "OFF")
        .define("BUILD_TOOLS", "OFF")
        .build();

    println!("cargo:rustc-link-lib=static=cubeb");
    if windows {
        println!("cargo:rustc-link-lib=dylib=avrt");
        println!("cargo:rustc-link-lib=dylib=ksuser");
        println!("cargo:rustc-link-lib=dylib=ole32");
        println!("cargo:rustc-link-lib=dylib=user32");
        println!("cargo:rustc-link-lib=dylib=winmm");
        println!("cargo:rustc-link-search=native={}/lib", dst.display());
    } else if darwin {
        println!("cargo:rustc-link-lib=framework=AudioUnit");
        println!("cargo:rustc-link-lib=framework=CoreAudio");
        println!("cargo:rustc-link-lib=framework=CoreServices");
        println!("cargo:rustc-link-lib=dylib=c++");
        println!("cargo:rustc-link-search=native={}/lib", dst.display());
    } else {
        if freebsd || android {
            println!("cargo:rustc-link-lib=dylib=c++");
        } else {
            println!("cargo:rustc-link-lib=dylib=stdc++");
        }
        println!("cargo:rustc-link-search=native={}/lib", dst.display());
        println!("cargo:rustc-link-search=native={}/lib64", dst.display());

        // Ignore the result of find_library. We don't care if the
        // libraries are missing.
        let _ = pkg_config::find_library("alsa");
        let _ = pkg_config::find_library("libpulse");
        let _ = pkg_config::find_library("jack");
        if android {
            println!("cargo:rustc-link-lib=dylib=OpenSLES");
        }
    }
}
