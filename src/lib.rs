use std::{ffi::{CStr, c_char}, fs::{self, File}, io::{self, Write}, path::Path};
use walkdir::WalkDir;
use zip::{self, ZipWriter, write::SimpleFileOptions};

#[unsafe(no_mangle)]
pub extern "C" fn extract_zip(input: *const c_char, output: *const c_char) -> i32 {
    unsafe {
        let input_zip = match CStr::from_ptr(input).to_str() {
            Ok(s) => s,
            Err(_) => return -2, // FLAG_BAD_PTR
        };

        let output_dir= match CStr::from_ptr(output).to_str() {
            Ok(d) => d,
            Err(_) => return -2 // FLAG_BAD_PTR
        };

        let file = match fs::File::open(input_zip) {
            Ok(f) => f,
            Err(_) => return -1, // FLAG_FILE_NOT_EXISTS
        };
        
        let mut archive = match zip::ZipArchive::new(file) {
            Ok(a) => a,
            Err(_) => return -3, // FLAG_NOT_ZIP
        };

        let output = std::path::Path::new(output_dir);

        for i in 0..archive.len() {
            let mut file = archive.by_index(i).unwrap();

            let output = match file.enclosed_name() {
                Some(path) => {
                    output.join(path)
                }

                None => continue
            };

            if file.is_dir() {
                let _ = fs::create_dir_all(output);
            } else {
                if let Some(p) = output.parent() {
                    if !p.exists() {
                        let _ = fs::create_dir_all(p);
                    }
                }

                let mut outfile = fs::File::create(&output).unwrap();
                let _ = io::copy(&mut file, &mut outfile);
            }
        }
    }

    3 // FLAG_OK
}

#[unsafe(no_mangle)]
pub extern "C" fn create_zip(input_folder: *const c_char, output: *const c_char) -> i32 {
    unsafe {
        let input = match CStr::from_ptr(input_folder).to_str() {
            Ok(s) => s,
            Err(_) => return -2, // FLAG_BAD_PTR
        };

        let output_zip= match CStr::from_ptr(output).to_str() {
            Ok(d) => d,
            Err(_) => return -2 // FLAG_BAD_PTR
        };

        let output_zip_file = match File::create(Path::new(output_zip)) {
            Ok(f) => f,
            Err(_) => return -1 // FLAG_FILE_NOT_EXISTS
        };

        let mut zip = ZipWriter::new(output_zip_file);
        let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        let walk = WalkDir::new(input);

        for entry in walk.into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            let name = path.strip_prefix(Path::new(input)).unwrap();

            if path.is_file() {
                let _ = zip.start_file(name.to_string_lossy(), options);
                let mut f = File::open(path).unwrap();
                let mut buffer = Vec::new();
                let _ = io::copy(&mut f, &mut buffer);
                zip.write_all(&buffer).unwrap();
            } else if !name.as_os_str().is_empty() {
                zip.add_directory(name.to_string_lossy(), options).unwrap();
            }
        }

        zip.finish().unwrap();
    }

    3 // FLAG_OK
}

#[unsafe(no_mangle)]
pub extern "C" fn extract_zip_pwd(
    input: *const c_char, 
    output: *const c_char, 
    password: *const c_char
) -> i32 {
    unsafe {
        let input_zip = match CStr::from_ptr(input).to_str() {
            Ok(s) => s,
            Err(_) => return -2, // FLAG_BAD_PTR
        };

        let output_dir= match CStr::from_ptr(output).to_str() {
            Ok(d) => d,
            Err(_) => return -2 // FLAG_BAD_PTR
        };

        let pwd = CStr::from_ptr(password).to_bytes();

        let file = fs::File::open(input_zip).expect("Could not open zip file");
        
        let mut archive = match zip::ZipArchive::new(file) {
            Ok(a) => a,
            Err(_) => return -3, // FLAG_NOT_ZIP
        };

        let output_path = std::path::Path::new(output_dir);

        for i in 0..archive.len() {
            let mut file = match archive.by_index_decrypt(i, pwd) {
                Ok(f) => f,
                Err(_) => {
                    continue;
                }
            };

            let out_file_path = match file.enclosed_name() {
                Some(path) => output_path.join(path),
                None => continue,
            };

            if file.is_dir() {
                let _ = fs::create_dir_all(&out_file_path);
            } else {
                if let Some(p) = out_file_path.parent() {
                    let _ = fs::create_dir_all(p);
                }
                let mut outfile = fs::File::create(&out_file_path).unwrap();
                let _ = io::copy(&mut file, &mut outfile);
            }
        }
    }

    3
}

#[unsafe(no_mangle)]
pub extern "C" fn create_zip_pwd(
    input_folder: *const c_char, 
    output: *const c_char, 
    password: *const c_char
) -> i32 {
    unsafe {
       let input = match CStr::from_ptr(input_folder).to_str() {
            Ok(s) => s,
            Err(_) => return -2, // FLAG_BAD_PTR
        };

        let output_zip= match CStr::from_ptr(output).to_str() {
            Ok(d) => d,
            Err(_) => return -2 // FLAG_BAD_PTR
        };

        let pwd = CStr::from_ptr(password).to_str().expect("Invalid password");

        let file = match File::create(Path::new(output_zip)) {
            Ok(f) => f,
            Err(_) => return -1 // FLAG_FILE_NOT_EXISTS
        };

        let mut zip = ZipWriter::new(file);
        
        let options = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .with_aes_encryption(zip::AesMode::Aes256, pwd);

        let walk = WalkDir::new(input);

        for entry in walk.into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            let name = path.strip_prefix(Path::new(input)).unwrap();

            if path.is_file() {
                zip.start_file(name.to_string_lossy(), options).unwrap();
                let mut f = File::open(path).unwrap();
                io::copy(&mut f, &mut zip).unwrap();
            } else if !name.as_os_str().is_empty() {
                zip.add_directory(name.to_string_lossy(), options).unwrap();
            }
        }

        zip.finish().unwrap();
    }

    3 // FLAG_OK
}

#[unsafe(no_mangle)]
pub extern "C" fn is_zip_encrypted(input: *const c_char) -> i32 {
    unsafe {
        let input_zip = match CStr::from_ptr(input).to_str() {
            Ok(s) => s,
            Err(_) => return -2, // FLAG_BAD_PTR
        };

        let file = match fs::File::open(input_zip) {
            Ok(f) => f,
            Err(_) => return -1, // FLAG_FILE_NOT_EXISTS
        };

        let mut archive = match zip::ZipArchive::new(file) {
            Ok(a) => a,
            Err(_) => return -3, // FLAG_NOT_ZIP
        };
        
        // проход по каждому файлику в архиве и проверка на то имеет ли он пароль
        for i in 0..archive.len() {
            if let Ok(file) = archive.by_index_raw(i) {
                if file.encrypted() {
                    return 1; // FLAG_NEEDS_PASSWORD
                }
            }
        }

        0
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn is_password_correct_for_zip(input: *const c_char, password: *const c_char) -> i32 {
    unsafe {
        let input_zip = match CStr::from_ptr(input).to_str() {
            Ok(s) => s,
            Err(_) => return -2 // FLAG_BAD_PTR
        };

        let password = match CStr::from_ptr(password).to_str() {
            Ok(s) => s,
            Err(_) => return -2 // FLAG_BAD_PTR
        };

        let file = match fs::File::open(input_zip) {
            Ok(f) => f,
            Err(_) => return -1 // FLAG_FILE_NOT_EXISTS
        };

        let mut archive = match zip::ZipArchive::new(file) {
            Ok(arc) => arc,
            Err(_) => return -3 // FLAG_NOT_ZIP
        };

        // мы должны проверить ток первый файл а не весь архив я полагаю
        if !archive.is_empty() {
            let is_encrypted = match archive.by_index_raw(0) {
                Ok(file) => file.encrypted(),
                Err(_) => return -5 // FLAG_PASSWORD_CORRECT
            };

            if is_encrypted {
                if let Ok(_) = archive.by_index_decrypt(0, password.as_bytes()) {
                    return -5 // FLAG_PASSWORD_CORRECT
                }
            }
        } else {
            return -4 // FLAG_ARCHIVE_EMPTY
        }
    }

    -6 // FLAG_PASSWORD_INCORRECT
}