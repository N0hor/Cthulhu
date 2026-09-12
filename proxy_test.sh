#!/bin/bash
HOST="http://localhost:8080"

echo ">>> T01 : GET / (no parameters)"
curl -s -o /dev/null "$HOST/"

echo ">>> T02 : GET /?a=1234&b=5678"
curl -s -o /dev/null "$HOST/?a=1234&b=5678"

echo ">>> T03 : GET /unknown (unknown path)"
curl -s -o /dev/null "$HOST/unknown"

echo ">>> T04 : POST / (method not allowed)"
curl -s -o /dev/null -X POST "$HOST/"

echo ">>> T05 : GET /send_msg_form (method not allowed)"
curl -s -o /dev/null "$HOST/send_msg_form"

echo ">>> T06 : GET /send_msg_form/ (trailing slash)"
curl -s -o /dev/null "$HOST/send_msg_form/"

echo ">>> T07 : GET /Send_Msg_Form (different case)"
curl -s -o /dev/null "$HOST/Send_Msg_Form"

echo ">>> T08 : GET /?a=9999 (upper limit)"
curl -s -o /dev/null "$HOST/?a=9999"

echo ">>> T09 : GET /?a=12345 (too long)"
curl -s -o /dev/null "$HOST/?a=12345"

echo ">>> T10 : GET /?a=12a3 (forbidden character)"
curl -s -o /dev/null "$HOST/?a=12a3"

echo ">>> T11 : GET /?a= (empty forbidden)"
curl -s -o /dev/null "$HOST/?a="

echo ">>> T12 : GET /?a=-1 (minus sign forbidden)"
curl -s -o /dev/null "$HOST/?a=-1"

echo ">>> T13 : GET /?a=%21 (URL-encoded '!' forbidden)"
curl -s -o /dev/null "$HOST/?a=%21"

echo ">>> T14 : GET /?a=12&a=99 (duplicate key)"
curl -s -o /dev/null "$HOST/?a=12&a=99"

echo ">>> T15 : GET /?c=abc (c not allowed for GET)"
curl -s -o /dev/null "$HOST/?c=abc"

echo ">>> T16 : GET /?b=42 (only b)"
curl -s -o /dev/null "$HOST/?b=42"

echo ">>> T17 : PUT /?c=abcdefgh (valid)"
curl -s -o /dev/null -X PUT "$HOST/?c=abcdefgh"

echo ">>> T18 : PUT /?c=ABCDEFGH (uppercase forbidden)"
curl -s -o /dev/null -X PUT "$HOST/?c=ABCDEFGH"

echo ">>> T19 : PUT /?c=abcdefghijk (11 chars > 10)"
curl -s -o /dev/null -X PUT "$HOST/?c=abcdefghijk"

echo ">>> T20 : PUT /?c=abc123 (digits forbidden)"
curl -s -o /dev/null -X PUT "$HOST/?c=abc123"

echo ">>> T21 : POST form msg=hello"
curl -s -o /dev/null -X POST -d 'msg=hello' "$HOST/send_msg_form"

echo ">>> T22 : POST form msg=hello_world-42"
curl -s -o /dev/null -X POST -d 'msg=hello_world-42' "$HOST/send_msg_form"

echo ">>> T23 : POST form msg=paff!!!! (forbidden character)"
curl -s -o /dev/null -X POST -d 'msg=paff!!!!' "$HOST/send_msg_form"

echo ">>> T24 : POST form msg= (empty)"
curl -s -o /dev/null -X POST -d 'msg=' "$HOST/send_msg_form"

echo ">>> T25 : POST form msg=hello&other=x (parameter not allowed)"
curl -s -o /dev/null -X POST -d 'msg=hello&other=x' "$HOST/send_msg_form"

echo ">>> T26 : POST form without body (no Content-Type)"
curl -s -o /dev/null -X POST "$HOST/send_msg_form"

echo ">>> T27 : POST form msg=<script> (raw angle brackets)"
curl -s -o /dev/null -X POST -d 'msg=<script>' "$HOST/send_msg_form"

echo ">>> T28 : POST form msg=%3Cscript%3E (URL-encoded)"
curl -s -o /dev/null -X POST -d 'msg=%3Cscript%3E' "$HOST/send_msg_form"

echo ">>> T29 : POST form Content-Type: text/plain"
curl -s -o /dev/null -X POST -H 'Content-Type: text/plain' -d 'msg=hello' "$HOST/send_msg_form"

echo ">>> T30 : POST form Content-Type: application/json"
curl -s -o /dev/null -X POST -H 'Content-Type: application/json' -d 'msg=hello' "$HOST/send_msg_form"

echo ">>> T31 : POST form Content-Type + charset"
curl -s -o /dev/null -X POST -H 'Content-Type: application/x-www-form-urlencoded; charset=UTF-8' -d 'msg=hello' "$HOST/send_msg_form"

echo ">>> T32 : POST form msg=60×'a' (too long)"
curl -s -o /dev/null -X POST -d "msg=$(printf 'a%.0s' {1..60})" "$HOST/send_msg_form"

echo ">>> T33 : POST JSON {\"msg\":\"yolo\"}"
curl -s -o /dev/null -X POST -H 'Content-Type: application/json' -d '{"msg":"yolo"}' "$HOST/send_msg_json"

echo ">>> T34 : POST JSON {\"msg\":\"paff!!!!\"}"
curl -s -o /dev/null -X POST -H 'Content-Type: application/json' -d '{"msg":"paff!!!!"}' "$HOST/send_msg_json"

echo ">>> T35 : POST JSON invalid {msg:\"yolo\"}"
curl -s -o /dev/null -X POST -H 'Content-Type: application/json' -d '{msg:"yolo"}' "$HOST/send_msg_json"

echo ">>> T36 : POST JSON {\"msg\":42}"
curl -s -o /dev/null -X POST -H 'Content-Type: application/json' -d '{"msg":42}' "$HOST/send_msg_json"

echo ">>> T37 : POST JSON {\"msg\":\"yolo\",\"extra\":\"x\"}"
curl -s -o /dev/null -X POST -H 'Content-Type: application/json' -d '{"msg":"yolo","extra":"x"}' "$HOST/send_msg_json"

echo ">>> T38 : POST JSON {} (empty object)"
curl -s -o /dev/null -X POST -H 'Content-Type: application/json' -d '{}' "$HOST/send_msg_json"

echo ">>> T39 : POST JSON {\"msg\":\"\"}"
curl -s -o /dev/null -X POST -H 'Content-Type: application/json' -d '{"msg":""}' "$HOST/send_msg_json"

echo ">>> T40 : POST JSON []"
curl -s -o /dev/null -X POST -H 'Content-Type: application/json' -d '[]' "$HOST/send_msg_json"

echo ">>> T41 : POST JSON null"
curl -s -o /dev/null -X POST -H 'Content-Type: application/json' -d 'null' "$HOST/send_msg_json"

echo ">>> T42 : POST JSON empty body"
curl -s -o /dev/null -X POST -H 'Content-Type: application/json' -d '' "$HOST/send_msg_json"

echo ">>> T43 : POST JSON Content-Type + charset"
curl -s -o /dev/null -X POST -H 'Content-Type: application/json; charset=utf-8' -d '{"msg":"yolo"}' "$HOST/send_msg_json"

echo ">>> T44 : POST JSON {\"msg\":\"ünicode\"}"
curl -s -o /dev/null -X POST -H 'Content-Type: application/json' -d '{"msg":"ünicode"}' "$HOST/send_msg_json"

echo ">>> T45 : POST form msg=hello%00world"
curl -s -o /dev/null -X POST -d 'msg=hello%00world' "$HOST/send_msg_form"

echo ">>> T46 : POST form msg=%E2%82%AC (euro)"
curl -s -o /dev/null -X POST -d 'msg=%E2%82%AC' "$HOST/send_msg_form"

echo ">>> T47 : POST form with stray query ?foo=bar (undeclared query)"
curl -s -o /dev/null -X POST -d 'msg=hello' "$HOST/send_msg_form?foo=bar"

echo ">>> T48 : POST JSON with stray query ?foo=bar (undeclared query)"
curl -s -o /dev/null -X POST -H 'Content-Type: application/json' -d '{"msg":"yolo"}' "$HOST/send_msg_json?foo=bar"

echo ">>> T49 : GET /?a=0000"
curl -s -o /dev/null "$HOST/?a=0000"

echo ">>> T50 : GET /?a=%30%31%32%33 (URL-encoded '0123')"
curl -s -o /dev/null "$HOST/?a=%30%31%32%33"

echo ">>> T51 : POST wq ?z=abc123 body msg=hello (all valid)"
curl -s -o /dev/null -X POST -d 'msg=hello' "$HOST/send_msg_with_query?z=abc123"

echo ">>> T52 : POST wq ?z=abc123 body msg=hello&msg2=x (msg2 not allowed)"
curl -s -o /dev/null -X POST -d 'msg=hello&msg2=x' "$HOST/send_msg_with_query?z=abc123"

echo ">>> T53 : POST wq ?z= body msg=hello (z empty)"
curl -s -o /dev/null -X POST -d 'msg=hello' "$HOST/send_msg_with_query?z="

echo ">>> T54 : POST wq ?z=ABC body msg=hello (uppercase OK)"
curl -s -o /dev/null -X POST -d 'msg=hello' "$HOST/send_msg_with_query?z=ABC"

echo ">>> T55 : POST wq ?z=21chars body msg=hello (z too long)"
curl -s -o /dev/null -X POST -d 'msg=hello' "$HOST/send_msg_with_query?z=abcdefghijklmnopqrstuvw"

echo ">>> T56 : POST wq ?z=abc-123 body msg=hello ('-' forbidden in z)"
curl -s -o /dev/null -X POST -d 'msg=hello' "$HOST/send_msg_with_query?z=abc-123"

echo ">>> T57 : POST wq without query body msg=hello (missing z)"
curl -s -o /dev/null -X POST -d 'msg=hello' "$HOST/send_msg_with_query"

echo ">>> T58 : POST wq ?z=abc123 body msg=paff!!!! (! forbidden in msg)"
curl -s -o /dev/null -X POST -d 'msg=paff!!!!' "$HOST/send_msg_with_query?z=abc123"

echo ">>> T59 : POST wq ?z=abc123&other=x body msg=hello (query parameter not allowed)"
curl -s -o /dev/null -X POST -d 'msg=hello' "$HOST/send_msg_with_query?z=abc123&other=x"

echo ">>> T60 : POST wq ?z=abc123 body JSON (content-type mismatch)"
curl -s -o /dev/null -X POST -H 'Content-Type: application/json' -d '{"msg":"hello"}' "$HOST/send_msg_with_query?z=abc123"