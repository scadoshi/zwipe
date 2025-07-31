# this creates a new user named pedro with email number1dog@email.com sets his password to woof

curl -X POST http://localhost:8080/api/v1/auth/register -H "Content-Type: application/json" -d '{"username": "pedro", "email": "number1dog@email.com", "password": "woof"}'


# authenticates as pedro

curl -X POST http://localhost:8080/api/v1/auth/login -H "Content-Type: application/json" -d '{"identifier": "pedro", "password": "woof"}'

# extra commands worth knowing

# -o curl_result.txt
# writes to a file called curl_result.txt
# -i 
# returns headers
# -v
# verbose=shows request and response
# -X
# indicates an http request - should be followed by POST, PUT, GET, etc.
# -H
# labels headers
# "Content-Type: appliction/json"
# standard label for json requests
# -d
# data=payload of request
# -s
# silent=no progress bars
# --json
# sets header to "Content-Type: application/json" with less words