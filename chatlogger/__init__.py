from chatlogger_core import Core
import json, os
from datetime import datetime
from uuid import UUID, uuid4 as uuid

PROMPT = 0
RESPONSE = 1
class Message:
    def __init__(self, text:str=None, idx:int=0, role:bool=None):
        self.is_response = int(role) if role else role # 1 (response) if role is true
        self.text = text
        self.depth = idx
    def __repr__(self):
        if self.text is None:
            return f"ROOT<{self.depth}>"
        if self.is_response == RESPONSE:
            return f"Response<{self.depth}>({self.text})"
        else:    
            return f"Prompt<{self.depth}>({self.text})"
    def __bool__(self):
        return True if self.text is not None else False
    
class Chat:
    def __init__(self, core, idx:int):
        self.core = core
        self.idx = idx
        self.current_idx = 0
        
    def __len__(self):
        """returns the number of active messages"""
        return self.core.lenchat(self.idx)
    def size(self)->int:
        """returns the number of total messages"""
        return self.core.sizechat(self.idx)
    def __getitem__(self, idx:int)->Message:
        assert idx>=0 and idx <= self.__len__()
        # return Chat(self.core, idx)
        msg = self.core.get_msg(self.idx, idx)
        if msg is not None:
            text, role, idx = msg
            return Message(text, idx, role)
        return Message()
    
    def history(self, pos:int|None=None, size:int|None=None)->list:
        messages = []
        for msg in self:
            messages.append(msg)
        if pos is None:
            pos = len(messages)
        if size is None:
            size = len(messages)
        return messages[max(0, pos-size):pos]
    
    def __iter__(self):
        for idx in range(1, self.__len__()+1):
            yield self[idx]
    
    def __repr__(self):
        a = self.core.reprchat(self.idx)
        return f"Chat<view=tree>(\n-----------------TREE-----------------\n{a}-----------------****-----------------\n)"
    def __str__(self):
        return f"Chat<size={self.size()}; len={self.__len__()}>({self.core.strchat(self.idx)})"
    

    
    
    def new_message(self, prompt, response):
        self.core.new_message(self.idx, prompt, response)
    def regenerate_last(self, response):
        self.core.regenerate_last(self.idx, response)
    def edit_last(self, prompt, response):
        self.core.edit_last(self.idx, prompt, response)
    def regenerate(self, pos:int, response:str):
        assert pos%2==0 and pos>0 and pos<=self.__len__()
        self.core.regenerate(self.idx, pos, response)
    def edit(self, pos:int, prompt:int, response:str):
        assert pos%2==1 and pos>0 and pos<=self.__len__()
        self.core.edit(self.idx, pos, prompt, response)
    def modify(self, pos:int, text:str):
        assert pos>0 and pos<=self.__len__()
        self.core.modify(self.idx, pos, text)
    
    # assert or try catch exception which one should be used...
    def curr_idx_response_last(self)->int:
        return self.core.curr_idx_response_last(self.idx)
    def curr_idx_prompt_last(self)->int:
        return self.core.curr_idx_prompt_last(self.idx)
    def curr_neighbours_prompt_last(self)->int:
        return self.core.curr_neighbours_prompt_last(self.idx)
    def curr_neighbours_response_last(self)->int:
        return self.core.curr_neighbours_response_last(self.idx)
    def prev_response_last(self):
        assert self.curr_idx_response_last() >= 1
        self.core.prev_response_last(self.idx)
    def next_response_last(self):
        assert self.curr_idx_response_last() < self.curr_neighbours_response_last()-1
        self.core.next_response_last(self.idx)
    def prev_prompt_last(self):
        assert self.curr_idx_prompt_last() >= 1
        self.core.prev_prompt_last(self.idx)
    def next_prompt_last(self):
        assert self.curr_idx_prompt_last() < self.curr_neighbours_prompt_last()-1
        self.core.next_prompt_last(self.idx)

    def curr_idx(self, pos:int)->int:
        """Returns the idx of self in all neighbours"""
        assert pos > 0 and pos<=self.__len__()
        return self.core.curr_idxchat(self.idx, pos)
    def curr_neighbours(self, pos:int)->int:
        """Returns the number of neighbours including self"""
        assert pos > 0 and pos<=self.__len__()
        return self.core.curr_neighbourschat(self.idx, pos)
    
    def set_next_idx(self, pos:int, newidx:int):
        assert pos >= 0 and pos<self.__len__()
        assert newidx >= 0 and newidx < self.curr_neighbours(pos+1)
        self.core.set_next_idxchat(self.idx, pos, newidx)
    def prev(self, pos:int):
        assert self.curr_idx(pos) >= 1
        self.core.prevchat(self.idx, pos)
    def next(self, pos:int):
        assert self.curr_idx(pos) < self.curr_neighbours(pos)-1
        self.core.nextchat(self.idx, pos)
class DataBase:
    def __init__(self, path=None):
        self.core = Core()
        if path is not None :
            if not os.path.exists(path):
                with open(path, 'w'): ...
                self.save(path)
            else:
                self.load(path)
        self.path = path
    @classmethod
    def new(cls, path:str):
        return cls(path)
    def commit(self):
        assert self.path is not None, "please set path while making database., db = DataBase.new(path)"
        self.save(self.path)
    def load(self, path:str):
        self.core.load(path)
    def save(self, path:str):
        self.core.save(path)
    def __len__(self):
        return self.core.len()
    def __getitem__(self, idx:int)->Chat:
        assert idx >= 0 and idx < self.core.len()
        return Chat(self.core, idx)
    def __iter__(self):
        for idx in range(self.__len__()):
            yield self[idx]
    def new_chat(self)->int:
        return self.core.new_chat()
    def size(self)->int:
        return self.core.size()
    def __repr__(self):
        return f"DataBase(len={self.__len__()}, messages={self.size()})"

class Memory:
    def __init__(self, size:int = 10, 
                 system_prompt="You are an intelligent assistant. You always provide well-reasoned answers that are both correct and helpful.", 
                 system_token="system", 
                 user_token="user", 
                 ai_token="assistant", newline="\n", separator=": "):
        """ The File Prompt will be in this format 
            system_prompt + separator + system_token + newline
            user_token + separator + prompt + newline
            ai_token + separator + response + newline
            ... n
            user_token + separator + prompt + newline
            ai_token
        """
        self.size = size
        self.system_prompt = system_prompt
        self.system_token = system_token
        self.user_token = user_token
        self.ai_token = ai_token
        self.separtor = separator
        self.newline = newline
    def history(self, chat:Chat, prompt:str|None=None, pos:int|None=None)->list:
        if pos is not None:
            if pos%2==1:
                assert prompt is None, "Prompt must be empty so i can use old prompt here"
            else:
                assert prompt is not None, "Prompt must be non empty"
        else: 
            assert prompt is not None
        history = [
            {"role": self.system_token, "content": self.system_prompt},
        ]
        for msg in chat.history(pos=pos, size=self.size):
            if msg.is_response == RESPONSE:
                history.append(
                    {"role": self.ai_token, "content": msg.text},
                )
            else:
                history.append(
                    {"role": self.user_token, "content": msg.text},
                )
        if prompt is not None:
            history.append(
                {"role": self.user_token, "content": prompt},
            )
        return history
        
    def raw(self, chat:Chat, prompt:str|None=None, pos:int|None=None)->str:
        if pos is not None:
            if pos%2==1:
                assert prompt is None, "Prompt must be empty so i can use old prompt here"
            else:
                assert prompt is not None, "Prompt must be non empty"
        else: 
            assert prompt is not None
            
        text = f"{self.system_prompt}{self.separtor}{self.system_token}{self.newline}"
        for msg in chat.history(pos=pos, size=self.size):
            if msg.is_response == RESPONSE:
                text += f"{self.ai_token}{self.separtor}{msg.text}{self.newline}"
            else:
                text += f"{self.user_token}{self.separtor}{msg.text}{self.newline}"
        if prompt is None:
            text += f"{self.ai_token}{self.separtor}"
        else:
            text += f"{self.user_token}{self.separtor}{prompt}{self.newline}{self.ai_token}{self.separtor}"
        return text

class User:
    def __init__(self, userid, table, db:DataBase):
        self.userid = userid
        self.username = table[userid]['username']
        self.table = table
        self.db = db
    def chats(self):
        return [
            (i, self.db[j['idx']], j['info'], j['time']) for i,j in self.table[self.userid]['chats'].items()
        ]
    def new_chat(self, prompt_info:str)->UUID:
        idx = self.db.new_chat()
        uid = uuid()
        self.table[self.userid]["chats"].update({
            str(uid) : {
            "idx": idx, 
            "time": datetime.now().isoformat(),
            "info": prompt_info
        }})
        return uid
    def __getitem__(self, uid:UUID|str):
        elem = self.table[self.userid]['chats'].get(str(uid), None)
        if elem is None: return elem
        return self.db[elem['idx']], elem['info'], elem['time']
    def __iter__(self):
        for i,j in self.table[self.userid]['chats'].items():
            yield (i, self.db[j['idx']], j['info'], j['time'])
    def __len__(self):
        return len(self.table[self.userid]['chats'])
    def __repr__(self) -> str:
        return f'User<{self.userid}; len={self.__len__()}>({self.username})'
    
class UserTable:
    def __init__(self, db:DataBase, path=None):
        self.path = path
        self.db = db
        self.table = {
            # unique userid => {username: name, chats: {(chat id 1), chat id 2, ...}}
        }
        if path is not None :
            if not os.path.exists(path): self.save(path)
            else: 
                try: self.load(path)
                except json.decoder.JSONDecodeError: self.save(path)
    def commit(self):
        assert self.path is not None
        self.save(self.path)
    def __getitem__(self, userid)->User:
        elem = self.table.get(userid, None)
        if elem is None: return elem
        return User(userid, self.table, self.db)
    
    def new_user(self, userid, username):
        data = self.table.get(userid, None)
        if data is None:
            self.table.update({
                userid: {
                    "username": username,
                    "chats": {
                        
                    }
                }
            })
    
    def save(self, path):
        with open(path, 'w') as f:
            json.dump(self.table, f)
    def load(self, path):
        with open(path, 'r') as f:
            self.table = json.load(f)
    def __repr__(self) -> str:
        return self.table.__repr__()

# if __name__ == '__main__':
#     db = DataBase()
#     # db.load(r"dataset.bin")
#     # db.save(r"main.bin")
#     # print(db)
#     # chat = db[2]
#     idx = db.new_chat()
#     chat = db[idx]
#     print(chat)
#     chat.new_message("Hello", "World!")
#     chat.regenerate_last("hey")
#     chat.edit_last("ss", "you!")
#     chat.edit_last("i", "you!")
#     chat.new_message("Hello", "World!")
#     chat.new_message("Hello", "World!")
#     chat.edit_last("i", "you!")
#     chat.prev_prompt_last()
#     chat.regenerate_last("hey2")
#     chat.next_prompt_last()
#     chat.prev_prompt_last()
#     chat.edit(3, "asd", "res")
#     chat.regenerate(2, "res2")
#     chat.modify(2, "ajasd")
#     chat.regenerate(2, "res2")
#     chat.prev(2)
#     chat.prev(2)
#     chat.prev(1)
#     chat.set_next_idx(0, 2)
#     print(repr(chat))
#     print(chat.curr_neighbours(1))
#     print(chat.__len__())
#     print(chat[0])
#     print(chat[1])
    