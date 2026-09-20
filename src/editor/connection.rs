use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use super::{EditorCommand, EditorEvent, Query, QueryId, QueryRequest, QueryResponse};
#[derive(Clone)]
pub struct EditorQueryClient { sender: Sender<QueryRequest>, next_id: Arc<AtomicU64> }
impl EditorQueryClient { pub fn request(&self, query: Query) -> QueryId { let id=self.next_id.fetch_add(1,Ordering::Relaxed); self.sender.send(QueryRequest{id,query}).expect("editor service query channel disconnected"); id } }
#[derive(Clone)]
pub struct EditorCommandClient { sender: Sender<EditorCommand> }
impl EditorCommandClient { pub fn send(&self, command: EditorCommand) { self.sender.send(command).expect("editor service command channel disconnected"); } }
pub struct EditorEventReceiver { receiver: Receiver<EditorEvent> }
impl EditorEventReceiver { pub fn try_recv(&self)->Option<EditorEvent>{self.receiver.try_recv().ok()} }
pub struct EditorConnection { pub queries: EditorQueryClient, pub commands: EditorCommandClient, pub events: EditorEventReceiver, responses: Receiver<QueryResponse> }
impl EditorConnection { pub fn new()->(Self,EditorServiceChannels){ let (query_tx,query_rx)=mpsc::channel(); let (response_tx,response_rx)=mpsc::channel(); let (command_tx,command_rx)=mpsc::channel(); let (event_tx,event_rx)=mpsc::channel(); (Self{queries:EditorQueryClient{sender:query_tx,next_id:Arc::new(AtomicU64::new(1))},commands:EditorCommandClient{sender:command_tx},events:EditorEventReceiver{receiver:event_rx},responses:response_rx},EditorServiceChannels{query_rx,response_tx,command_rx,event_tx}) } pub fn try_recv_response(&self)->Option<QueryResponse>{self.responses.try_recv().ok()} }
pub struct EditorServiceChannels { pub query_rx: Receiver<QueryRequest>, pub response_tx: Sender<QueryResponse>, pub command_rx: Receiver<EditorCommand>, pub event_tx: Sender<EditorEvent> }
