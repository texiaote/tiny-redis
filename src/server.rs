use futures::{SinkExt, StreamExt};
use tokio::{select, signal};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;
use tokio::sync::broadcast::Receiver;
use tokio_util::codec::Framed;

use crate::cmd::Cmd;
use crate::codec::{LineCodec, RedisCodec, RedisFrame};
use crate::connection::Connection;
use crate::db::{Db, SharedDb};
use crate::RedisResult;

#[derive(Debug)]
pub struct Server {
    listener: TcpListener,

    notify_shutdown: broadcast::Sender<()>,
}

impl Server {
    pub fn new(listener: TcpListener) -> Self {

        // send shutdown to all active connections
        // broadcast channel for this purpose
        // a receiver is needed, the subscribe() method on ther sender is needed
        let (notify_shutdown, _) = broadcast::channel(1);

        Self {
            listener,
            notify_shutdown,
        }
    }

    //这里只能执行一次
    pub async fn run(self) -> RedisResult<()> {


        //启动数据库，并且传入一个命令接受功能，随时准备接收关闭信号的命令
        let shared_db = Db::new(self.notify_shutdown.subscribe());
        loop {
            select! {

                Ok((socket,_)) = self.listener.accept() =>{
                    let sharedDb = shared_db.clone();

                    let notify_shutdown = self.notify_shutdown.subscribe();
                    tokio::spawn(async move {
                        let _ = process(socket, sharedDb, notify_shutdown).await;
                    });
                }
                _= signal::ctrl_c()=>{

                    println!("结束信号");
                    break;
                }

            }
        }
        drop(self.notify_shutdown);
        Ok(())
    }
}

async fn process(socket: TcpStream, mut db: SharedDb, mut notify_shutdown: Receiver<()>) -> RedisResult<()> {

    // 将stream信息转换成编码
    let mut framed = Framed::new(socket, RedisCodec);

    loop {
        if let Some(Ok(RedisFrame::Array(arr))) = framed.next().await {

            // 生成command




        } else {
            let errorFrame = RedisFrame::Error(" invalid command".to_string());

            framed.send(errorFrame).await.unwrap();
        }
    }


    // //读取二进制信息，将内容转换成Frame信息
    // let mut connection = Connection::new(socket);
    //
    // loop {
    //     select! {
    //         result=handle(&mut connection,&mut db) => {
    //
    //             println!("result");
    //         }
    //         _=notify_shutdown.recv()=>{
    //             println!("任务结束，连接即将关闭");
    //             break;
    //         }
    //     }
    // }


    Ok(())
}


// async fn handle(connection: &mut Connection, db: &mut SharedDb) -> RedisResult<()> {
//     if let Some(frame) = connection.read_frame().await? {
//
//         // 执行命令，
//         //将命令转变为指令
//         let command: Cmd = frame.try_into()?;
//
//         let frame = command.execute(db).await?;
//         connection.write_frame(frame).await?;
//     }
//     Ok(())
// }


