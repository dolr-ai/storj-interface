# proper input
POST {{host}}/move-to-nsfw
Authorization: Bearer testtoken
{
  "publisher_user_id": "{{publisher}}",
  "video_id": "7ec40a0b9aba4307a97e8666822ed563"
}
HTTP 200

# Without auth token
POST {{host}}/move-to-nsfw
{
  "publisher_user_id": "{{publisher}}",
  "video_id": "7ec40a0b9aba4307a97e8666822ed563"
}
HTTP 401

# With wrong token
POST {{host}}/move-to-nsfw
Authorization: Bearer wrongtoken
{
  "publisher_user_id": "{{publisher}}",
  "video_id": "7ec40a0b9aba4307a97e8666822ed563"
}
HTTP 401

# malformed input
POST {{host}}/move-to-nsfw
Authorization: Bearer testtoken
`malformed json input`
HTTP 415

# missing required information
POST {{host}}/move-to-nsfw
Authorization: Bearer testtoken
{
  "video_id": "7ec40a0b9aba4307a97e8666822ed563"
}
HTTP 422
