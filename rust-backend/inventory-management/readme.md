update a product's quantity by sending a post request to with a body like this "{
  "product_id": "22222222-2222-2222-2222-222222222222",
  "quantity_change": -12
}
" with this header "/inventory/{supplier_id}/update", 

post or create a new product by sending a post request to this route "http://localhost:3002/inventory" with this body structure "{
  "product_id": "33333333-3333-3333-3333-333333333333",
  "supplier_id": "11111111-1111-1111-1111-111111111111",
  "quantity": 50,
  "name": "Rice bag",
  "low_stock_threshold": 10,
  "unit": "bags"
}". 

get all the product from a singular supplier by sending a get request to this route ".route("/inventory/{supplier_id}", web::get().to(handlers::get_inventory))"

