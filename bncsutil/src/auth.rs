use std::path::Path;

use crate::*;

pub fn auth_check(
    is_tft: bool,
    war3_version: u32,
    war3_path: &Path,
    key_roc: String,
    key_tft: String,
    value_string_formula: String,
    mpq_file_name: String,
    client_token: u32,
    server_token: u32,
) -> Option<(u32, u64, Vec<u8>, Vec<u8>, String)> {
    let w3exe = war3_path.join("Warcraft III.exe");
    let w3exe2 = war3_path.join("warcraft.exe");
    let w3exe3 = war3_path.join("war3.exe");

    let file_war3_exe = if w3exe.exists() {
        w3exe
    } else if w3exe2.exists() {
        w3exe2
    } else {
        w3exe3
    };

    let is_exist = file_war3_exe.exists();

    dbg!(is_exist);

    if !is_exist {
        println!("[BNCSUI] unable to open [{}]", file_war3_exe.display());

        return None;
    }

    // todotodo: check getExeInfo return value to ensure 1024 bytes was enough

    let (_, exe_info, exe_version) = get_exe_info(file_war3_exe.as_path());
    let mpq_ver = extract_mpq_number(mpq_file_name.to_string()).unwrap();

    // for war3version <= 28, we use war3.exe, storm.dll, and game.dll
    // for war3version == 29, we use Warcraft III.exe only
    let exe_version_hash = if war3_version <= 28 {
        let file_storm_dll = if war3_path.join("Storm.dll").exists() {
            war3_path.join("Storm.dll")
        } else {
            war3_path.join("storm.dll")
        };

        let file_game_dll = war3_path.join("game.dll");

        if !file_storm_dll.exists() {
            println!("[BNCSUI] unable to open [{}]", file_storm_dll.display());
        }

        if !file_game_dll.exists() {
            println!("[BNCSUI] unable to open [{}]", file_game_dll.display());
        }

        check_revision_flat(
            value_string_formula,
            file_war3_exe.as_path(),
            file_storm_dll.as_path(),
            file_game_dll.as_path(),
            mpq_ver,
        )
    } else {
        let files = vec![file_war3_exe.as_path()];
        check_revision(value_string_formula, files, mpq_ver)
    };

    let key_info_roc = create_key_info(key_roc, client_token, server_token);

    if is_tft {
        let key_info_tft = create_key_info(key_tft, client_token, server_token);

        if key_info_roc.len() == 36 && key_info_tft.len() == 36 {
            return Some((
                exe_version,
                exe_version_hash,
                key_info_roc,
                key_info_tft,
                exe_info,
            ));
        } else {
            println!("[BNCSUI] unable to create ROC key info - invalid ROC key");
            println!("[BNCSUI] unable to create TFT key info - invalid TFT key");
        }
    }

    if key_info_roc.len() == 36 {
        return Some((
            exe_version,
            exe_version_hash,
            key_info_roc,
            vec![],
            exe_info,
        ));
    } else {
        println!("[BNCSUI] unable to create ROC key info - invalid ROC key");
    }

    return None;
}

#[cfg(test)]
mod auth_tests {
    use super::*;

    #[test]
    fn test_auth_check() {
        // logonType: '2 0 0 0',
        // serverToken: '23 6 226 51',
        // mpqFileTime: '128 105 207 135 198 27 214 1',
        // ix86VerFileName: 'ver-IX86-1.mpq',
        // valueString: 'A=1239576727 C=1604096186 B=4198521212 4 A=A+S B=B-C C=C^A A=A+B'
        let is_tft = true;
        let key_roc = "FFFFFFFFFFFFFFFFFFFFFFFFFF".to_string();
        let key_tft = "FFFFFFFFFFFFFFFFFFFFFFFFFF".to_string();
        let client_token: u32 = 130744796;
        let server_token: u32 = 2115470359;
        let war3_version = 28;
        let war3_path = Path::new("../mock/");
        let value_string_formula =
            "A=1239576727 C=1604096186 B=4198521212 4 A=A+S B=B-C C=C^A A=A+B".to_string();
        let mpq_file_name = "ver-IX86-1.mpq".to_string();

        let (exe_version, exe_version_hash, key_info_roc, key_info_tft, exe_info) = auth_check(
            is_tft,
            war3_version,
            war3_path,
            key_roc,
            key_tft,
            value_string_formula,
            mpq_file_name,
            client_token,
            server_token,
        )
        .unwrap();

        dbg!(exe_version_hash);
        dbg!(key_info_roc);
        dbg!(key_info_tft);

        assert_eq!(exe_version, 18613504);
        assert_eq!(exe_info, "war3.exe 02/02/24 12:39:34 562152".to_string());
    }
}
