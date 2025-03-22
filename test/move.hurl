# proper input
POST {{host}}/move-to-nsfw
Authorization: Bearer {{api_token}}
{
  "publisher_user_id": "{{publisher}}",
  "video_id": "{{video_id}}"
}
HTTP 200

# Without auth token
POST {{host}}/move-to-nsfw
{
  "publisher_user_id": "{{publisher}}",
  "video_id": "{{video_id}}"
}
HTTP 401

# With wrong token
POST {{host}}/move-to-nsfw
Authorization: Bearer wrongtoken
{
  "publisher_user_id": "{{publisher}}",
  "video_id": "{{video_id}}"
}
HTTP 401

# malformed input
POST {{host}}/move-to-nsfw
Authorization: Bearer {{api_token}}
`malformed json input`
HTTP 415

# missing required information
POST {{host}}/move-to-nsfw
Authorization: Bearer {{api_token}}
{
  "video_id": "{{video_id}}"
}
HTTP 422
