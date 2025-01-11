# Debt tracker API

This is a simple API to track debts.

## Description

This API allows you to create, read, update and delete debts.

## Endpoints

### Create a debt

```
POST /add
```

#### Request body

```json
{
  "name": "John Doe",
  "amount": 100.0,
}
```

#### Response body
```json
{
    "_id": "$oid",
    "name": "String",
    "amount": "isize",
    "createdAt": "DateTime",
    "updatedAt": "DateTime",
}
```


