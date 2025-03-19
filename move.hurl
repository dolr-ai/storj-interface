# proper input
POST {{host}}/move-to-nsfw
{
  "publisher_user_id": "{{publisher}}",
  "video_id": "7ec40a0b9aba4307a97e8666822ed563"
}
HTTP 200

# malformed input
POST {{host}}/duplicate
`malformed json input`
HTTP 415

# missing required information
POST {{host}}/duplicate
{
  "video_id": "7ec40a0b9aba4307a97e8666822ed563"
}
HTTP 422
