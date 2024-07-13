from flask import Flask, redirect, url_for, request, render_template, json, jsonify, Response
from chatlogger import DataBase, Chat, Message, RESPONSE, PROMPT, Memory, User, UserTable
from openai import OpenAI
import requests

BASE_URL = 'http://localhost:1234/v1'
assert requests.get(BASE_URL).status_code == 200, f"Please check the base URL again... {BASE_URL}"
# todo ;- add like and dislike features... or just log them out from main db using history = 3 etc...

app = Flask(__name__)
client = OpenAI(base_url=BASE_URL, api_key="lm-studio")

memory = Memory(
    system_token="system",
    system_prompt="You are an intelligent assistant. You always provide well-reasoned answers that are both correct and helpful.",
    user_token="user",
    ai_token="assistant"
)
db = DataBase("database.bin")
user_table = UserTable(db, "user_table.bin")
user_table.new_user(userid="userid1", username="laksh")
user = user_table["userid1"]
# uid = user.new_chat("test 1")
# chat, info, timestamp = user[uid]

def llm(chat:Chat, prompt=None, pos=None)->str:
    history = memory.history(chat, prompt=prompt, pos=pos)
    completion = client.chat.completions.create(
        model="model-identifier",
        messages=history,
        temperature=0.7,
        stream=True,
    )
    new_message = ""
    for chunk in completion:
        if chunk.choices[0].delta.content:
            chk = chunk.choices[0].delta.content
            print(chk, end="", flush=True)
            new_message += chk
    return new_message
def llm_stream(chat:Chat, prompt:str|None=None, pos:int|None=None):
    history = memory.history(chat, prompt=prompt, pos=pos)
    completion = client.chat.completions.create(
        model="model-identifier",
        messages=history,
        temperature=0.7,
        stream=True,
    )
    for chunk in completion:
        if chunk.choices[0].delta.content:
            yield chunk.choices[0].delta.content
    
@app.route('/')
def home():
    chatList = []
    for uid, _, info, timestamp in user:
        chatList.append([str(uid), info, timestamp])
    return render_template('index.html', chatList=chatList)

@app.route('/api/llm/', methods=['POST'])
def api_llm():
    data = json.loads(request.data)
    def generate():
        if data.get('new_message', False):
            prompt = data['prompt']
            chatidx = data['chatidx']
            # chat = db[chatidx]
            chat, _, _ = user[chatidx]
            fn = lambda : llm_stream(chat, prompt)
            update = lambda response : chat.new_message(prompt, response)
        elif data.get('regenerate', False):
            pos = data['depth']
            chatidx = data['chatidx']
            # chat = db[chatidx]
            chat, _, _ = user[chatidx]
            fn = lambda : llm_stream(chat, pos=pos-1)
            update = lambda response : chat.regenerate(pos, response)
        elif data.get('edit', False):
            pos = data['depth']
            chatidx = data['chatidx']
            # chat = db[chatidx]
            chat, _, _ = user[chatidx]
            prompt = data['prompt']
            fn = lambda : llm_stream(chat, prompt, pos-1)
            update = lambda response : chat.edit(pos, prompt, response)
        
        
        response = ""
        for chunk in fn():
            response += chunk
            yield chunk
        update(response)
        db.commit()
        user_table.commit()
    return Response(generate(), mimetype='text/plain')
    
@app.route('/api/v1/', methods=['POST'])
def api_v1():
    data = json.loads(request.data)
    try:
        if data.get('new_chat', False):
            prompt = data['prompt']
            uid = user.new_chat(prompt_info=prompt)
            return jsonify({
                'OK': True,
                'ID': str(uid)
            })
        elif data.get('prev', False):
            pos = data['depth']
            chatidx = data['chatidx']
            chat, _, _ = user[chatidx]
            chat.prev(pos)
            # db[chatidx].prev(pos)
        elif data.get('next', False):
            pos = data['depth']
            chatidx = data['chatidx']
            chat, _, _ = user[chatidx]
            chat.next(pos)
            # db[chatidx].next(pos)
        db.commit()
        user_table.commit()
        return jsonify({
            'OK': True,
        })
    except Exception as e:
        print("ERROR", e)
        return jsonify({
            'OK': False,
        })

@app.route('/c/<idx>')
def chat(idx):
    if user[idx] is None: return "Not Found <a href='/'>Go Home</a>"
    chatList = []
    for uid, _, info, timestamp in user:
        # active if idx == uid ...
        chatList.append([str(uid), info, timestamp])
    
    messages = []
    chat, _, _ = user[idx]
    for i in range(1, len(chat)+1): 
        msg: Message = chat[i]
        messages.append((msg.depth, msg.is_response==RESPONSE, msg.text, chat.curr_idx(i)+1, chat.curr_neighbours(i)))
    
    return render_template('chat.html', chatList=chatList, messages=messages, chatidx=idx)

if __name__ == '__main__':
    app.run(debug=True)