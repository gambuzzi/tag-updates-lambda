import requests

menu_url = "https://api.airtable.com/v0/appFsXK6zp1O2SyV8/tblHWvgCofYT2Puac"
tags_url = "https://api.airtable.com/v0/appFsXK6zp1O2SyV8/tblC2T0WC2MTR3MQ3"
headers = {
    "Authorization": "Bearer <secret>"
}

menu_tags = set()
offset = "0"
records = []
while True:
    response = requests.get(menu_url, params={"offset": offset}, headers=headers).json()
    records.extend(response["records"])
    try:
        offset = response["offset"]
    except KeyError:
        break
for record in records:
    for tag in record["fields"]["tags"]:
        menu_tags.add(tag.strip())

remote_tags = {}
offset = "0"
max_sort_order = 0
while True:
    response = requests.get(tags_url, params={"offset": offset}, headers=headers).json()
    for r in response["records"]:
        remote_tags[r["fields"]["tag"]] = r
        if r["fields"]["sort_order"] > max_sort_order:
            max_sort_order = r["fields"]["sort_order"]
    try:
        offset = response["offset"]
    except KeyError:
        break

# create new tags
for tag in menu_tags:
    if tag not in remote_tags:
        max_sort_order += 10
        data = {"records": [{"fields": {"tag": tag, "sort_order": max_sort_order}}]}
        response = requests.post(tags_url, json=data, headers=headers)
        if "DEBUG" == "DEBUG":
            print(response.status_code)
