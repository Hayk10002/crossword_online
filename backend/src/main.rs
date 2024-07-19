use std::{collections::HashMap, net::IpAddr, path::PathBuf};

use rocket::{fs::NamedFile, response::status::NotFound, tokio::sync::Mutex, State};

#[macro_use]
extern crate rocket;

#[derive(Default)]
struct UserData(Mutex<HashMap<IpAddr, u32>>);
impl UserData
{
    async fn register_new_user(&self, new_user_ip_addr: &IpAddr) -> bool
    {
        let mut l = self.0.lock().await;
        if l.contains_key(new_user_ip_addr) { false }
        else { l.insert(*new_user_ip_addr,  0); true }
    }

    async fn login(&self, user_ip_addr: &IpAddr)
    {
        let mut l = self.0.lock().await;
        if !l.contains_key(user_ip_addr) { drop(l); self.register_new_user(user_ip_addr).await; }
        else { *l.get_mut(user_ip_addr).expect("impossible") += 1; }
    }
}

async fn get_index() -> Result<NamedFile, NotFound<String>>
{
    NamedFile::open("../frontend/dist/index.html").await
        .map_err(|e| NotFound(e.to_string()))
}

#[get("/<path..>")]
async fn index(path: PathBuf, ip_addr: IpAddr, user_data: &State<UserData>) -> Result<NamedFile, NotFound<String>>
{
    user_data.login(&ip_addr).await;
    let path = PathBuf::from("../frontend/dist").join(path);
    match NamedFile::open(path.as_path()).await
    {
        Ok(file) => Ok(file),
        Err(_) => get_index().await,
    }
}

#[get("/data/<path..>")]
async fn data(path: PathBuf) -> Result<NamedFile, NotFound<String>>
{
    let path = PathBuf::from("data/").join(path);
    match NamedFile::open(path.as_path()).await
    {
        Ok(file) => Ok(file),
        Err(_) => get_index().await,
    }
}

#[launch]
fn rocket() -> _
{
    let user_data = UserData::default();

    rocket::build()
        .mount("/", routes![index, data])
        .manage(user_data)
}