from chatlogger_core import Core

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
        self.path = path
    @classmethod
    def new(cls, path:str):
        return cls(path)
    def commit(self):
        assert self.path is not None
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
    def new_chat(self)->Chat:
        return self.core.new_chat()
    def size(self)->int:
        return self.core.size()
    def __repr__(self):
        return f"DataBase(len={self.__len__()}, messages={self.size()})"

class Memory:
    def __init__(self, chat: Chat, size:int = 10, system_prompt="SYSTEM: ", system_token="SYSTEM: ", user_token="USER: ", ai_token="AI: ", newline="\n"):
        """ The File Prompt will be in this format 
            system_prompt + system_token + newline
            user_token + prompt + newline
            ai_token + response + newline
            ... n
            user_token + prompt + newline
            ai_token
        """
        self.chat = chat
        self.size = size
        self.system_prompt = system_prompt
        self.system_token = system_token
        self.user_token = user_token
        self.ai_token = ai_token
        self.newline = newline
    def __call__(self, prompt:str|None=None, pos:int|None=None)->str:
        if pos is not None:
            if pos%2==1:
                assert prompt is None, "Prompt must be empty so i can use old prompt here"
            else:
                assert prompt is not None, "Prompt must be non empty"
        else: 
            assert prompt is not None
            
        text = f"{self.system_prompt}{self.system_token}{self.newline}"
        for msg in self.chat.history(pos=pos, size=self.size):
            if msg.is_response == RESPONSE:
                text += f"{self.ai_token}{msg.text}{self.newline}"
            else:
                text += f"{self.user_token}{msg.text}{self.newline}"
        if prompt is None:
            prompt = self.chat[pos+1]
            text += f"{self.ai_token}"
        else:
            text += f"{self.user_token}{prompt}{self.newline}{self.ai_token}"
        return text

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
    