# tests/fixtures/adversarial/python/flask_route.py

from flask import Flask, request

app = Flask(__name__)

# ⚠️ This looks dead (no internal callers) but is a Flask route
@app.route('/api/v1/users', methods=['GET'])
def get_users():
    return {"users": []}

# ⚠️ This looks dead but is a route with path parameters
@app.route('/api/v1/users/<int:user_id>', methods=['GET'])
def get_user(user_id):
    return {"user_id": user_id}

# ⚠️ This looks dead but is a POST route
@app.route('/api/v1/users', methods=['POST'])
def create_user():
    data = request.json
    return {"created": True}

# ⚠️ This looks dead but is a PUT route
@app.route('/api/v1/users/<int:user_id>', methods=['PUT'])
def update_user(user_id):
    data = request.json
    return {"updated": user_id}

# ⚠️ This looks dead but is a DELETE route
@app.route('/api/v1/users/<int:user_id>', methods=['DELETE'])
def delete_user(user_id):
    return {"deleted": user_id}

if __name__ == '__main__':
    app.run()
