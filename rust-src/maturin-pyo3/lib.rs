use pyo3::prelude::*;
use std::{fmt::format, fs::File, io::{Read, Write}};

enum Role {
    Prompt,
    Response,
}

struct Message {
    text: String,
    idx: usize,
    role: Role,
}
impl Message {
    fn new(text: String, idx: usize, role: Role) -> Self {
        Self { text, idx, role }
    }
}
impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Role::Prompt => write!(f, "Prompt"),
            Role::Response => write!(f, "Response"),
        }
    }
}
impl std::fmt::Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}<{}>({})", self.role, self.idx, self.text)
    }
}
impl From<bool> for Role {
    fn from(value: bool) -> Self {
        if value{
            Self::Response
        }else{
            Self::Prompt
        }
    }
}

struct MessageTree{
    data: Option<Message>, // for root it is none
    children: Vec<MessageTree>,
    next_idx: usize,
}
impl MessageTree {
    fn new(text: String, idx: usize, role: Role)->Self{
        Self { data: Some(Message::new(text, idx, role)), children: Vec::<Self>::new(), next_idx: 0 }
    }
    fn new_root() -> Self {
        Self { data: None, children: Vec::<Self>::new(), next_idx: 0 }
    }
    fn new_tmp() -> Self {
        Self { data: None, children: Vec::<Self>::new(), next_idx: usize::MAX }
    }
    fn add_child(&mut self, child: MessageTree) {
        self.children.push(child);
    }
    fn __size(&self) -> usize {
        let mut size = 1;
        if self.children.len() != 0 {
            for msg in &self.children{
                size += msg.__size();
            }
        }
        size
    }
    fn size(&self) -> usize {
        self.__size() - 1
    }
    fn __print_helper(node: &MessageTree, level: usize, prefix: &str, isactive: bool)->String{
        let mut old_str: String = String::new();
        if level == 0 {
            if let Some(msg) = node.data.as_ref() { 
                panic!() // not implemented for non root instance
            }else{
                old_str += "Root\n";
                // print!("Root\n");
            }
        }else{
            for _ in 1..level{
                old_str += "    ";
                // print!("    ");
            }
            if isactive{
                old_str += format!("\x1b[96m{}\x1b[0m", prefix).as_str();
                // print!("\x1b[96m{}\x1b[0m", prefix);
            }else{
                old_str += format!("{}", prefix).as_str();
                // print!("{}", prefix);
            }
            if let Some(msg) = node.data.as_ref() { 
                if isactive{
                    old_str += format!("\x1b[96m{}\x1b[0m\n", msg).as_str();
                    // println!("\x1b[96m{}\x1b[0m", msg);
                }else{
                    old_str += format!("{}\n", msg).as_str();
                    // println!("{}", msg);
                }
            }
        }
        if node.children.len() != 0 {
            for idx in 0..node.children.len()-1{
                old_str += if idx == node.next_idx{
                    MessageTree::__print_helper(&node.children[idx], level + 1, "├──\x1b[92m*\x1b[0m", isactive)
                }else{
                    MessageTree::__print_helper(&node.children[idx], level + 1, "├── ", false)
                }.as_str()
            }
            old_str += if node.children.len()-1 == node.next_idx{
                MessageTree::__print_helper(&node.children[node.children.len()-1], level + 1, "└──\x1b[92m*\x1b[0m", isactive)
            }else{
                MessageTree::__print_helper(&node.children[node.children.len()-1], level + 1, "└── ", false)
            }.as_str()
        }
        old_str
    }
}
impl Iterator for MessageTree {
    type Item = Message;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(data) = self.data.take() { // for root do nothing
            return Some(data);
        }
        // Check children recursively
        let next_idx = self.next_idx;
        if next_idx < self.children.len(){ // check for a end
            let child = &mut self.children[self.next_idx];
            if let Some(msg) = child.next() {
                return Some(msg);
            }
        }
        None
    }
}
impl std::fmt::Display for MessageTree {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", MessageTree::__print_helper(self, 0, "└── ", true))
    }
}

struct Chats{
    root: MessageTree
}
impl Chats{
    fn new()->Self{
        Self { root: MessageTree::new_root() }
    }
    fn new_message(&mut self, prompt: String, response: String){
        let mut root = &mut self.root;
        let mut idx: usize = 1;
        while root.children.len() > 0 {
            root = &mut root.children[root.next_idx];
            idx += 1;
        }
        let mut child = MessageTree::new(prompt, idx, Role::Prompt);
        child.add_child(MessageTree::new(response,idx+1, Role::Response));
        root.add_child(child);
    }
    fn edit_last(&mut self, prompt: String, response: String){
        let mut root = &mut self.root;
        loop {
            root = &mut root.children[root.next_idx];

            let break_prompt = &mut root.children[root.next_idx];
            let break_response = &mut break_prompt.children[break_prompt.next_idx];
            if break_response.children.len() == 0{
                if let Some(msg) = root.data.as_mut() {
                    let idx = msg.idx + 1;
                    let mut prompt = MessageTree::new(prompt, idx, Role::Prompt);
                    let response = MessageTree::new(response, idx+1, Role::Response);
                    prompt.add_child(response);
                    root.next_idx = root.children.len();
                    root.add_child(prompt);
                    break;
                } else {
                    panic!()
                }
            }
        }
    }
    fn regenerate_last(&mut self, response: String){
        let mut root = &mut self.root;
        assert!(root.children.len() > 0);
        loop {
            let last = &mut root.children[root.next_idx];
            if last.children.len() == 0 {
                if let Some(msg) = root.data.as_mut() { // call disable for non root
                    let idx = msg.idx + 1;
                    let response = MessageTree::new(response, idx, Role::Response);
                    root.next_idx = root.children.len();
                    root.add_child(response);
                    break;
                } else {
                    panic!();
                }
            }
            root = &mut root.children[root.next_idx];
        }
    }
    fn regenerate(&mut self, idx: usize, response: String) {
        assert!(idx%2 == 0);
        let mut root = &mut self.root;
        for i in 0..idx-1{
            assert!(root.children.len() > 0);
            root = &mut root.children[root.next_idx];
        }
        if let Some(msg) = root.data.as_mut() { // call disable for non root
            let idx = msg.idx + 1;
            let response = MessageTree::new(response, idx, Role::Response);
            root.next_idx = root.children.len();
            root.add_child(response);
        }else {
            panic!();
        }
    }
    fn edit(&mut self, idx: usize, prompt: String, response: String) {
        assert!(idx%2 == 1);
        let mut root = &mut self.root;
        for i in 0..idx-1{
            assert!(root.children.len() > 0);
            root = &mut root.children[root.next_idx];
        }
        if let Some(msg) = root.data.as_mut() { // call disable for non root
            let idx = msg.idx + 1;
            let mut prompt = MessageTree::new(prompt, idx, Role::Prompt);
            let response = MessageTree::new(response, idx+1, Role::Response);
            prompt.add_child(response);
            root.next_idx = root.children.len();
            root.add_child(prompt);
        }else {
            let idx = 1;
            let mut prompt = MessageTree::new(prompt, idx, Role::Prompt);
            let response = MessageTree::new(response, idx+1, Role::Response);
            prompt.add_child(response);
            root.next_idx = root.children.len();
            root.add_child(prompt);
        }
    }
    fn modify(&mut self, idx:usize, text: String){
        let mut root = &mut self.root;
        for _ in 0..idx{
            assert!(root.children.len() > 0);
            root = &mut root.children[root.next_idx];
        }
        if let Some(msg) = root.data.as_mut() { // call disable for non root
            msg.text = text;
        }else {
            panic!();
        }        
    }

    fn add_child(&mut self, text: String, role: Role, idx: usize){
        // add children and activate it
        let mut root = &mut self.root;
        for i in 0..idx-1{
            assert!(root.children.len() > 0);
            root = &mut root.children[root.next_idx];
        }

        if let Some(msg) = root.data.as_mut() { // call disable for non root
            let idx = msg.idx + 1;
            let response = MessageTree::new(text, idx, role);
            root.next_idx = root.children.len();
            root.add_child(response);
        }else {
            let response = MessageTree::new(text, 1, role);
            root.next_idx = root.children.len();
            root.add_child(response);
        }
    }
    fn __set_next_idxs(node: &mut MessageTree, idxs: Vec<usize>, mut i:usize)->usize{
        let next_idx = idxs[i];
        node.next_idx = next_idx;
        i+=1;
        let mut result: usize = 1;
        for child in &mut node.children{
            let num_children = Chats::__set_next_idxs(child, idxs.clone(), i);
            i+=num_children;
            result+=num_children;
        }
        return result;
    }
    fn set_next_idxs(&mut self, idxs: Vec<usize>){
        let node = &mut self.root;
        Chats::__set_next_idxs(node, idxs, 0);
    }
    fn __get_next_idxs(node: &mut MessageTree, idxs: &mut Vec<usize>){
        idxs.push(node.next_idx);
        for child in &mut node.children{
            Chats::__get_next_idxs(child, idxs);
        }
    }
    
    fn get_next_idxs(&mut self)->Vec<usize>{
        let mut idxs = Vec::<usize>::new();
        let node = &mut self.root;
        Chats::__get_next_idxs(node, &mut idxs);
        println!("{:?}, len={}", idxs, idxs.len());
        idxs
    }
    fn curr_neighbours_prompt_last(&mut self) -> usize{
        let mut root = &mut self.root;
        assert!(root.children.len() > 0);
        loop {
            root = &mut root.children[root.next_idx];

            let break_prompt = &mut root.children[root.next_idx];
            let break_response = &mut break_prompt.children[break_prompt.next_idx];
            if break_response.children.len() == 0{
                return root.children.len();
            }
        }
    }
    fn curr_neighbours_response_last(&mut self) -> usize{
        let mut root = &mut self.root;
        assert!(root.children.len() > 0);
        loop {
            let last = &mut root.children[root.next_idx];
            if last.children.len() == 0 {
                return root.children.len();
            }
            root = &mut root.children[root.next_idx];
        }
    }
    fn curr_idx_prompt_last(&mut self) -> usize{
        let mut root = &mut self.root;
        assert!(root.children.len() > 0);
        loop {
            root = &mut root.children[root.next_idx];

            let break_prompt = &mut root.children[root.next_idx];
            let break_response = &mut break_prompt.children[break_prompt.next_idx];
            if break_response.children.len() == 0{
                return root.next_idx;
            }
        }
    }
    fn curr_idx_response_last(&mut self) -> usize{
        let mut root = &mut self.root;
        assert!(root.children.len() > 0);
        loop {
            let last = &mut root.children[root.next_idx];
            if last.children.len() == 0 {
                return root.next_idx;
            }
            root = &mut root.children[root.next_idx];
        }
    }
    fn next_prompt_last(&mut self){
        let mut root = &mut self.root;
        assert!(root.children.len() > 0);
        loop {
            root = &mut root.children[root.next_idx];

            let break_prompt = &mut root.children[root.next_idx];
            let break_response = &mut break_prompt.children[break_prompt.next_idx];
            if break_response.children.len() == 0{
                assert!(root.next_idx+1<root.children.len());
                root.next_idx = root.next_idx+1;
                break;
            }
        }
    }
    fn prev_prompt_last(&mut self){
        let mut root = &mut self.root;
        assert!(root.children.len() > 0);
        loop {
            root = &mut root.children[root.next_idx];

            let break_prompt = &mut root.children[root.next_idx];
            let break_response = &mut break_prompt.children[break_prompt.next_idx];
            if break_response.children.len() == 0{
                assert!(root.next_idx>0);
                root.next_idx = root.next_idx-1;
                break;
            }
        }
    }
    fn next_response_last(&mut self){
        let mut root = &mut self.root;
        assert!(root.children.len() > 0);
        loop {
            let last = &mut root.children[root.next_idx];
            if last.children.len() == 0 {
                assert!(root.next_idx+1<root.children.len());
                root.next_idx = root.next_idx+1;
                break;
            }
            root = &mut root.children[root.next_idx];
        }
    }
    fn prev_response_last(&mut self){
        let mut root = &mut self.root;
        assert!(root.children.len() > 0);
        loop {
            let last = &mut root.children[root.next_idx];
            if last.children.len() == 0 {
                assert!(root.next_idx>0);
                root.next_idx = root.next_idx-1;
                break;
            }
            root = &mut root.children[root.next_idx];
        }
    }
    fn next(&mut self, idx: usize){
        let mut root = &mut self.root;
        assert!(root.children.len() > 0);
        for _ in 0..idx-1{
            assert!(root.children.len() > 0);
            root = &mut root.children[root.next_idx];
        }
        assert!(root.next_idx+1<root.children.len());
        root.next_idx = root.next_idx+1;
    }
    fn prev(&mut self, idx: usize){
        let mut root = &mut self.root;
        assert!(root.children.len() > 0);
        for _ in 0..idx-1{
            assert!(root.children.len() > 0);
            root = &mut root.children[root.next_idx];
        }
        assert!(root.next_idx>0);
        root.next_idx = root.next_idx-1;
    }
    fn set_next_idx(&mut self, idx: usize, newidx: usize){
        let mut root = &mut self.root;
        for _ in 0..idx{
            assert!(root.children.len() > 0);
            root = &mut root.children[root.next_idx];
        }
        assert!(newidx>=0 && newidx<root.children.len());
        root.next_idx = newidx;
    }
    fn curr_idx(&mut self, idx: usize)->usize{
        let mut root = &mut self.root;
        if idx == 0{
            return root.next_idx;
        }
        for _ in 0..idx-1{
            assert!(root.children.len() > 0);
            root = &mut root.children[root.next_idx];
        }
        root.next_idx
    }
    fn curr_neighbours(&mut self, idx: usize)->usize{
        let mut root = &mut self.root;
        if idx == 0{
            return root.children.len()
        }
        for _ in 0..idx-1{
            assert!(root.children.len() > 0);
            root = &mut root.children[root.next_idx];
        }
        root.children.len()
    }
    fn get_msg(&mut self, idx: usize)->Option<(String, bool, usize)>{
        let mut root = &mut self.root;
        for _ in 0..idx{
            assert!(root.children.len() > 0);
            root = &mut root.children[root.next_idx];
        }
        if let Some(msg) = root.data.as_ref(){
            let msg_role: bool;
            match msg.role {
                Role::Prompt => msg_role = false,
                Role::Response => msg_role = true,
            };
            return Some((msg.text.clone(), msg_role, msg.idx));
        }else{
            return None;
        }
    }
}

struct Dataset {
    chats: Vec<Chats>,
}
impl Dataset {
    fn new()->Self{
        Self { chats: Vec::<Chats>::new() }
    }
    fn new_chat(&mut self) -> usize{
        let idx = self.chats.len();
        self.chats.push(Chats::new());
        idx
    }
    fn get(&mut self, idx: usize)->&mut Chats{
        assert!(idx < self.chats.len(), "chats index out of range");
        return &mut self.chats[idx];
    }
    fn len(&self) -> usize{
        return self.chats.len();
    }
    fn add(&mut self, chat: Chats){
        self.chats.push(chat);
    }
    fn save_helper(mut file:&File, node: &MessageTree, level: usize){
        if level == 0 {
            // print!("LEN : {}\n", node.size());
            file.write(&node.size().to_ne_bytes());
            file.write(&node.next_idx.to_ne_bytes());
        }else{
            // print!("{} | ", level);
            file.write(&level.to_ne_bytes());
            if let Some(msg) = node.data.as_ref() { 
                let raw_msg = msg.text.as_bytes();
                match msg.role {
                    Role::Prompt => file.write(&[0]),
                    Role::Response => file.write(&[1])
                };
                file.write(&node.next_idx.to_ne_bytes());
                file.write(&raw_msg.len().to_ne_bytes());
                file.write(&raw_msg);
                // print!("LEN: {} |", raw_msg.len());
                // if isactive{
                //     println!("\x1b[96m{}<{}>({})\x1b[0m", msg.role, msg.idx, msg.text);
                // }else{
                //     println!("{}<{}>({})", msg.role, msg.idx, msg.text);
                // }
            }
        }
        for idx in 0..node.children.len(){
            Dataset::save_helper(file, &node.children[idx], level + 1)
        }
    }
    fn save(&self, path: &str){
        let file = File::create(path).unwrap();
        for chat in self.chats.iter() {
            let root = &chat.root;
            Dataset::save_helper(&file, root, 0);
        }
    }
    fn load_helper(buffer:&Vec<u8>, mut pos: usize)->(usize, Chats){
        let bytes = [buffer[pos], buffer[pos+1], buffer[pos+2], buffer[pos+3], buffer[pos+4], buffer[pos+5], buffer[pos+6], buffer[pos+7]];
        let size = usize::from_ne_bytes(bytes);
        pos += 8;
        let mut chat = Chats::new();
        let mut idxs = Vec::<usize>::with_capacity(size+1);
        let bytes = [buffer[pos], buffer[pos+1], buffer[pos+2], buffer[pos+3], buffer[pos+4], buffer[pos+5], buffer[pos+6], buffer[pos+7]];
        let next_idx = usize::from_ne_bytes(bytes);
        idxs.push(next_idx);
        pos += 8;
        for _ in 0..size{
            let bytes = [buffer[pos], buffer[pos+1], buffer[pos+2], buffer[pos+3], buffer[pos+4], buffer[pos+5], buffer[pos+6], buffer[pos+7]];
            let depth = usize::from_ne_bytes(bytes);
            let role: Role = (buffer[pos+8] != 0).into();
            let bytes = [buffer[pos+9], buffer[pos+10], buffer[pos+11], buffer[pos+12], buffer[pos+13], buffer[pos+14], buffer[pos+15], buffer[pos+16]];
            let next_idx = usize::from_ne_bytes(bytes);
            idxs.push(next_idx);
            let bytes = [buffer[pos+17], buffer[pos+18], buffer[pos+19], buffer[pos+20], buffer[pos+21], buffer[pos+22], buffer[pos+23], buffer[pos+24]];
            let len = usize::from_ne_bytes(bytes);
            let text: String = String::from_utf8_lossy(&buffer[pos+25..pos+25+len]).to_string();
            chat.add_child(text, role, depth);
            pos += 25+len;
        }
        chat.set_next_idxs(idxs);
        (pos, chat)
    }
    fn load(path: &str)->Self{
        let mut file = File::open(path).unwrap();
        let mut buffer = Vec::<u8>::new();
        file.read_to_end(&mut buffer).unwrap();
        let mut result = Self::new();
        let mut pos = 0;
        while pos<buffer.len(){
            let (newpos, chat) = Dataset::load_helper(&buffer, pos);
            result.add(chat);
            pos = newpos;
        }
        result
    }
}

impl std::fmt::Display for Chats {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut root = &self.root;
        if root.children.len() == 0{
            return write!(f, "Chat()");
        }else{
            print!("Chat(\n");
        }
        while root.children.len() > 0 {
            if let Some(msg) = root.data.as_ref() { // call disable for non root
                println!("\t{}", msg);
            }
            root = &root.children[root.next_idx];
        }
        if let Some(msg) = root.data.as_ref() { // call disable for non root
            println!("\t{}", msg);
        }
        return write!(f, ")");
    }
}


#[pyclass]
struct Core{
    db: Dataset,
}
#[pymethods]
impl Core {
    #[new]
    fn new() -> Self {
        Self{db: Dataset::new()}
    }
    fn load(&mut self, path: String) {
        self.db = Dataset::load(&path);
    }
    fn save(&self, path: String) {
        self.db.save(&path);
    }
    fn new_chat(&mut self) -> usize{
        self.db.new_chat()
    }
    fn new_message(&mut self, idx:usize, prompt: String, response: String) {
        self.db.get(idx).new_message(prompt, response);
    }
    fn regenerate_last(&mut self, idx:usize, response: String) {
        self.db.get(idx).regenerate_last(response);
    }
    fn edit_last(&mut self, idx:usize, prompt: String, response: String) {
        self.db.get(idx).edit_last(prompt, response);
    }
    fn regenerate(&mut self, idx:usize, pos:usize, response: String) {
        self.db.get(idx).regenerate(pos, response);
    }
    fn edit(&mut self, idx:usize, pos:usize, prompt: String, response: String) {
        self.db.get(idx).edit(pos, prompt, response);
    }
    fn modify(&mut self, idx:usize, pos:usize, text:String){
        self.db.get(idx).modify(pos, text);   
    }

    fn prev_response_last(&mut self, idx:usize){
        self.db.get(idx).prev_response_last();
    }
    fn next_response_last(&mut self, idx:usize){
        self.db.get(idx).next_response_last();
    }
    fn prev_prompt_last(&mut self, idx:usize){
        self.db.get(idx).prev_prompt_last();
    }
    fn next_prompt_last(&mut self, idx:usize){
        self.db.get(idx).next_prompt_last();
    }
    fn curr_idx_response_last(&mut self, idx:usize)->usize{
        self.db.get(idx).curr_idx_response_last()
    }
    fn curr_idx_prompt_last(&mut self, idx:usize)->usize{
        self.db.get(idx).curr_idx_prompt_last()
    }
    fn curr_neighbours_prompt_last(&mut self, idx:usize)->usize{
        self.db.get(idx).curr_neighbours_prompt_last()
    }
    fn curr_neighbours_response_last(&mut self, idx:usize)->usize{
        self.db.get(idx).curr_neighbours_response_last()
    }

    fn prevchat(&mut self, idx:usize, pos:usize){
        self.db.get(idx).prev(pos);
    }
    fn nextchat(&mut self, idx:usize, pos:usize){
        self.db.get(idx).next(pos);
    }
    fn set_next_idxchat(&mut self, idx:usize, pos:usize, newidx:usize){
        self.db.get(idx).set_next_idx(pos, newidx);
    }
    fn curr_idxchat(&mut self, idx:usize, pos:usize)->usize{
        self.db.get(idx).curr_idx(pos)
    }
    fn curr_neighbourschat(&mut self, idx:usize, pos:usize)->usize{
        self.db.get(idx).curr_neighbours(pos)
    }
    fn get_msg(&mut self, idx:usize, pos:usize)->Option<(String, bool, usize)>{
        self.db.get(idx).get_msg(pos)
    }

    fn lenchat(&mut self, idx:usize) -> usize {
        let mut root = &self.db.get(idx).root;
        if root.children.len() == 0{
            return 0;
        }
        let mut size = 0;
        while root.children.len() > 0 {
            size+=1;
            root = &root.children[root.next_idx];
        }
        size
    } 
    fn reprchat(&mut self, idx:usize) -> String {
        let root = &self.db.get(idx).root;
        format!("{}", root)
    }
    fn strchat(&mut self, idx:usize)->String{
        let mut root = &self.db.get(idx).root;
        if root.children.len() == 0{
            return format!("");
        }
        let mut s = String::from("\n");
        while root.children.len() > 0 {
            if let Some(msg) = root.data.as_ref() { // call disable for non root
                s += format!("\t{}\n", msg).as_str();
            }
            root = &root.children[root.next_idx];
        }
        if let Some(msg) = root.data.as_ref() { // call disable for non root
            s += format!("\t{}\n", msg).as_str();
        }
        s
    }
    fn sizechat(&mut self, idx:usize) -> usize{
        self.db.get(idx).root.size()
    }
    fn size(&self) -> usize{
        let mut size = 0;
        for chat in self.db.chats.iter(){
            size+=chat.root.size();
        }
        size
    }
    fn len(&self) -> usize{
        self.db.len()
    }
}

/// A Python module implemented in Rust.
#[pymodule]
fn LLMChatLogCore(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Core>()?;
    Ok(())
}
