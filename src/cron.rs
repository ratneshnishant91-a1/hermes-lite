use crate::{store::{CronJob, Store}, tools::Tools};
use anyhow::Result;
use chrono::Utc;
use uuid::Uuid;

pub fn parse_interval(input: &str) -> Result<i64> {
    let s=input.trim().to_lowercase();
    if s=="hourly" { return Ok(3600); } if s=="daily" { return Ok(86400); }
    let p:Vec<_>=s.strip_prefix("every ").ok_or_else(|| anyhow::anyhow!("use daily, hourly, or every <n>[s|m|h]"))?.split_whitespace().collect();
    if p.len()!=1 { anyhow::bail!("use every <n>[s|m|h]"); }
    let (number,unit)=p[0].split_at(p[0].len()-1); let n:i64=number.parse()?;
    Ok(n * match unit { "s"=>1,"m"=>60,"h"=>3600,_=>anyhow::bail!("unit must be s, m, or h") })
}
pub fn add(store:&Store,name:&str,schedule:&str,command:&str)->Result<CronJob>{let every_secs=parse_interval(schedule)?;let next_run=Utc::now().timestamp()+every_secs;let job=CronJob{id:Uuid::new_v4().to_string(),name:name.into(),every_secs,command:command.into(),next_run,last_run:None,enabled:true};store.save_cron(&job)?;Ok(job)}
pub fn tick(store:&Store,tools:&Tools)->Result<usize>{let now=Utc::now().timestamp();let jobs=store.due_cron(now)?;for mut job in jobs.iter().cloned(){let _=tools.run_shell(&job.command);job.last_run=Some(now);job.next_run=now+job.every_secs;store.save_cron(&job)?;}Ok(jobs.len())}
