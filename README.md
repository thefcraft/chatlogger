# chatlogger

chatlogger is a simple Tree based database to store chats logs for a large language model. to save ollama or any other llm app chat history

Install Using
`pip install chatlogger-db`


DEMO
```python
from chatlogger-db import DataBase, Message, Chat, RESPONSE, PROMPT
if __name__ == '__main__':
    db = DataBase()
    # db.load(r"dataset.bin")
    # db.save(r"main.bin")
    # print(db)
    # chat = db[2]
    idx = db.new_chat()
    chat = db[idx]
    print(chat)
    chat.new_message("Hello", "World!")
    chat.regenerate_last("hey")
    chat.edit_last("ss", "you!")
    chat.edit_last("i", "you!")
    chat.new_message("Hello", "World!")
    chat.new_message("Hello", "World!")
    chat.edit_last("i", "you!")
    chat.prev_prompt_last()
    chat.regenerate_last("hey2")
    chat.next_prompt_last()
    chat.prev_prompt_last()
    chat.edit(3, "asd", "res")
    chat.regenerate(2, "res2")
    chat.modify(2, "ajasd")
    chat.regenerate(2, "res2")
    chat.prev(2)
    chat.prev(2)
    chat.prev(1)
    chat.set_next_idx(0, 2)
    print(repr(chat))
    print(chat.curr_neighbours(1))
    print(chat.__len__())
    print(chat[0])
    print(chat[1])
```