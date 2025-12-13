**10 sample `CreateProductRequest` bodies** using teh struct.
Each contains a **flexible `description` object** following your general schema.

---

# ✅ **1. Laptop**

```json
{
  "product_id": null,
  "supplier_id": "c2d1bcd7-77f2-4e52-8efe-0849d1a4f912",
  "name": "Ultrabook Pro 14",
  "description": {
    "name": "Ultrabook Pro 14",
    "unit_price": 950000,
    "quantity": 1,
    "description": "Lightweight high-performance laptop for professionals.",
    "currency": "NGN",
    "category": "Electronics",
    "sku": "LP-UB14-2025",
    "subtotal": 950000,
    "discount": {
      "type": "seasonal",
      "amount": 50000
    },
    "specifications": {
      "color": "Silver",
      "size": "14 inches",
      "weight": "1.2kg",
      "volume": null,
      "storage": "512GB SSD",
      "warranty": "2 years",
      "ram": "16GB",
      "material": "Aluminum",
      "other_specs": {}
    },
    "shipping": {
      "delivery_type": "doorstep",
      "method": "GIG Logistics",
      "station_location": null,
      "estimated_delivery_days": 3,
      "estimated_ready_in_hours": 24,
      "shipping_fee": 4500
    },
    "logistics": {
      "requires_heavy_transport": false,
      "truck_type": null,
      "offloading_required": false
    },
    "tax": {
      "vat_percentage": 7.5,
      "vat_amount": 71250
    },
    "final_total": 971750,
    "metadata": {
      "notes": "Handle with care",
      "delivery_instructions": "Call on arrival",
      "gift_wrapping": false
    }
  },
  "category": "Electronics",
  "price": 950000,
  "unit": "piece",
  "quantity": 10,
  "available": true,
  "low_stock_threshold": 3
}
```

---

# ✅ **2. Pack of Bottled Water**

```json
{
  "product_id": null,
  "supplier_id": "8ba9b5ee-42c1-4e0d-a81c-9880dbea41dd",
  "name": "Premium Bottled Water (12 Pack)",
  "description": {
    "name": "Premium Bottled Water",
    "unit_price": 1500,
    "quantity": 12,
    "description": "Clean and purified drinking water.",
    "currency": "NGN",
    "category": "Groceries",
    "sku": "BW-12PK-001",
    "subtotal": 1500,
    "discount": {
      "type": "bulk",
      "amount": 200
    },
    "specifications": {
      "color": null,
      "size": "75cl per bottle",
      "weight": null,
      "volume": "9L total",
      "storage": "Room temperature",
      "warranty": null,
      "ram": null,
      "material": "Plastic bottles",
      "other_specs": {}
    },
    "shipping": {
      "delivery_type": "pickup_station",
      "method": "Local Dispatch",
      "station_location": "Ikeja Pickup Hub",
      "estimated_delivery_days": 1,
      "estimated_ready_in_hours": 3,
      "shipping_fee": 500
    },
    "logistics": {
      "requires_heavy_transport": false,
      "truck_type": null,
      "offloading_required": false
    },
    "tax": {
      "vat_percentage": 0,
      "vat_amount": 0
    },
    "final_total": 1800,
    "metadata": {
      "notes": null,
      "delivery_instructions": null,
      "gift_wrapping": false
    }
  },
  "category": "Groceries",
  "price": 1500,
  "unit": "pack",
  "quantity": 100,
  "available": true,
  "low_stock_threshold": 10
}
```

---

# ✅ **3. Office Chair**

```json
{
  "product_id": null,
  "supplier_id": "5f7dc492-5e12-4f16-ab6b-606e983b5634",
  "name": "Ergonomic Mesh Office Chair",
  "description": {
    "name": "Ergonomic Mesh Office Chair",
    "unit_price": 65000,
    "quantity": 1,
    "description": "Adjustable ergonomic chair for office comfort.",
    "currency": "NGN",
    "category": "Furniture",
    "sku": "OF-MSH-01",
    "subtotal": 65000,
    "discount": {
      "type": "none",
      "amount": 0
    },
    "specifications": {
      "color": "Black",
      "size": null,
      "weight": "6kg",
      "volume": null,
      "storage": null,
      "warranty": "1 year",
      "ram": null,
      "material": "Mesh + Steel",
      "other_specs": {}
    },
    "shipping": {
      "delivery_type": "doorstep",
      "method": "DHL",
      "station_location": null,
      "estimated_delivery_days": 5,
      "estimated_ready_in_hours": 48,
      "shipping_fee": 8500
    },
    "logistics": {
      "requires_heavy_transport": true,
      "truck_type": "Small van",
      "offloading_required": true
    },
    "tax": {
      "vat_percentage": 7.5,
      "vat_amount": 4875
    },
    "final_total": 78375,
    "metadata": {
      "notes": "Assembly required",
      "delivery_instructions": null,
      "gift_wrapping": false
    }
  },
  "category": "Furniture",
  "price": 65000,
  "unit": "piece",
  "quantity": 25,
  "available": true,
  "low_stock_threshold": 5
}
```

---

# ✅ **4. Running Shoes**

```json
{
  "product_id": null,
  "supplier_id": "b77d77fb-0dfe-4bc4-8c3d-4c5210b5abc1",
  "name": "AeroRun Sports Shoes",
  "description": {
    "name": "AeroRun Sports Shoes",
    "unit_price": 32000,
    "quantity": 1,
    "description": "Breathable lightweight running shoes.",
    "currency": "NGN",
    "category": "Clothing",
    "sku": "SH-AR-32",
    "subtotal": 32000,
    "discount": {
      "type": "voucher",
      "amount": 2000
    },
    "specifications": {
      "color": "Blue",
      "size": "43",
      "weight": null,
      "volume": null,
      "storage": null,
      "warranty": "6 months",
      "ram": null,
      "material": "Synthetic mesh",
      "other_specs": {}
    },
    "shipping": {
      "delivery_type": "doorstep",
      "method": "FedEx",
      "station_location": null,
      "estimated_delivery_days": 4,
      "estimated_ready_in_hours": 12,
      "shipping_fee": 2500
    },
    "logistics": {
      "requires_heavy_transport": false,
      "truck_type": null,
      "offloading_required": false
    },
    "tax": {
      "vat_percentage": 7.5,
      "vat_amount": 2400
    },
    "final_total": 33900,
    "metadata": {
      "notes": null,
      "delivery_instructions": "Leave with security",
      "gift_wrapping": true
    }
  },
  "category": "Clothing",
  "price": 32000,
  "unit": "pair",
  "quantity": 40,
  "available": true,
  "low_stock_threshold": 6
}
```

---

# ✅ **5. Smartphone**

```json
{
  "product_id": null,
  "supplier_id": "e6f99d0f-8a30-4dee-a8c0-eceadc1dfb91",
  "name": "Smart X12 5G",
  "description": {
    "name": "Smart X12 5G",
    "unit_price": 485000,
    "quantity": 1,
    "description": "Flagship smartphone with 5G and AMOLED display.",
    "currency": "NGN",
    "category": "Electronics",
    "sku": "PH-X12-5G",
    "subtotal": 485000,
    "discount": {
      "type": "wholesale",
      "amount": 15000
    },
    "specifications": {
      "color": "Graphite",
      "size": "6.5 inches",
      "weight": "178g",
      "volume": null,
      "storage": "256GB",
      "warranty": "1 year",
      "ram": "12GB",
      "material": "Glass & metal",
      "other_specs": {}
    },
    "shipping": {
      "delivery_type": "doorstep",
      "method": "UPS",
      "station_location": null,
      "estimated_delivery_days": 2,
      "estimated_ready_in_hours": 6,
      "shipping_fee": 3000
    },
    "logistics": {
      "requires_heavy_transport": false,
      "truck_type": null,
      "offloading_required": false
    },
    "tax": {
      "vat_percentage": 7.5,
      "vat_amount": 36375
    },
    "final_total": 509375,
    "metadata": {
      "notes": "Includes free screen protector",
      "delivery_instructions": null,
      "gift_wrapping": false
    }
  },
  "category": "Electronics",
  "price": 485000,
  "unit": "piece",
  "quantity": 15,
  "available": true,
  "low_stock_threshold": 4
}
```
Here are the **remaining 5 sample `CreateProductRequest` bodies**, bringing the total to 10.

Each one uses the same flexible description schema and realistic product details.

---

# ✅ **6. Electric Blender**

```json
{
  "product_id": null,
  "supplier_id": "0f1c6c88-4238-4c07-b52f-d8bd9129b901",
  "name": "PowerBlend 900",
  "description": {
    "name": "PowerBlend 900",
    "unit_price": 23000,
    "quantity": 1,
    "description": "High-speed 900W blender suitable for smoothies and grinding.",
    "currency": "NGN",
    "category": "Home Appliances",
    "sku": "HB-PB900-01",
    "subtotal": 23000,
    "discount": {
      "type": "seasonal",
      "amount": 1000
    },
    "specifications": {
      "color": "White",
      "size": null,
      "weight": "2kg",
      "volume": "1.5L jar",
      "storage": null,
      "warranty": "1 year",
      "ram": null,
      "material": "Plastic + steel blades",
      "other_specs": {}
    },
    "shipping": {
      "delivery_type": "doorstep",
      "method": "Dispatch Rider",
      "station_location": null,
      "estimated_delivery_days": 2,
      "estimated_ready_in_hours": 4,
      "shipping_fee": 1500
    },
    "logistics": {
      "requires_heavy_transport": false,
      "truck_type": null,
      "offloading_required": false
    },
    "tax": {
      "vat_percentage": 7.5,
      "vat_amount": 1725
    },
    "final_total": 24725,
    "metadata": {
      "notes": "Comes with extra grinder cup",
      "delivery_instructions": null,
      "gift_wrapping": false
    }
  },
  "category": "Home Appliances",
  "price": 23000,
  "unit": "piece",
  "quantity": 50,
  "available": true,
  "low_stock_threshold": 8
}
```

---

# ✅ **7. Wooden Dining Table**

```json
{
  "product_id": null,
  "supplier_id": "61bc9485-d51a-41b1-a4b4-4fdb27f7da3e",
  "name": "Classic 6-Seater Dining Table",
  "description": {
    "name": "Classic 6-Seater Dining Table",
    "unit_price": 210000,
    "quantity": 1,
    "description": "Premium hardwood dining table with polished finish.",
    "currency": "NGN",
    "category": "Furniture",
    "sku": "DT-WD-6ST",
    "subtotal": 210000,
    "discount": {
      "type": "none",
      "amount": 0
    },
    "specifications": {
      "color": "Walnut Brown",
      "size": "6-seater",
      "weight": "25kg",
      "volume": null,
      "storage": null,
      "warranty": "2 years",
      "ram": null,
      "material": "Hardwood",
      "other_specs": {}
    },
    "shipping": {
      "delivery_type": "doorstep",
      "method": "Truck Delivery",
      "station_location": null,
      "estimated_delivery_days": 7,
      "estimated_ready_in_hours": 48,
      "shipping_fee": 15000
    },
    "logistics": {
      "requires_heavy_transport": true,
      "truck_type": "Large Truck",
      "offloading_required": true
    },
    "tax": {
      "vat_percentage": 7.5,
      "vat_amount": 15750
    },
    "final_total": 241750,
    "metadata": {
      "notes": "Assembly services available",
      "delivery_instructions": "Ensure space at entrance",
      "gift_wrapping": false
    }
  },
  "category": "Furniture",
  "price": 210000,
  "unit": "piece",
  "quantity": 8,
  "available": true,
  "low_stock_threshold": 2
}
```

---

# ✅ **8. Protein Powder**

```json
{
  "product_id": null,
  "supplier_id": "b3e4cd71-c50d-40c1-b971-848433efc3df",
  "name": "MuscleMax Whey Protein (2kg)",
  "description": {
    "name": "MuscleMax Whey Protein",
    "unit_price": 45000,
    "quantity": 1,
    "description": "High-quality whey protein for muscle building.",
    "currency": "NGN",
    "category": "Health & Fitness",
    "sku": "WP-MMX-2KG",
    "subtotal": 45000,
    "discount": {
      "type": "voucher",
      "amount": 5000
    },
    "specifications": {
      "color": null,
      "size": "2kg container",
      "weight": "2kg",
      "volume": null,
      "storage": "Store in cool dry place",
      "warranty": null,
      "ram": null,
      "material": "Plastic container",
      "other_specs": {
        "flavour": "Vanilla"
      }
    },
    "shipping": {
      "delivery_type": "pickup_station",
      "method": "GIG Pickup",
      "station_location": "Lekki Phase 1 Station",
      "estimated_delivery_days": 2,
      "estimated_ready_in_hours": 8,
      "shipping_fee": 1200
    },
    "logistics": {
      "requires_heavy_transport": false,
      "truck_type": null,
      "offloading_required": false
    },
    "tax": {
      "vat_percentage": 7.5,
      "vat_amount": 3375
    },
    "final_total": 41575,
    "metadata": {
      "notes": "Includes measuring scoop",
      "delivery_instructions": null,
      "gift_wrapping": false
    }
  },
  "category": "Health & Fitness",
  "price": 45000,
  "unit": "piece",
  "quantity": 35,
  "available": true,
  "low_stock_threshold": 5
}
}
```

---

# ✅ **9. Wristwatch**

```json
{
  "product_id": null,
  "supplier_id": "7fcd3536-c0fc-4a0c-98e8-6e75b9b170db",
  "name": "ChronoMaster Leather Watch",
  "description": {
    "name": "ChronoMaster Leather Watch",
    "unit_price": 78000,
    "quantity": 1,
    "description": "Elegant men’s wristwatch with leather strap.",
    "currency": "NGN",
    "category": "Accessories",
    "sku": "WT-CHR-LTHR",
    "subtotal": 78000,
    "discount": {
      "type": "none",
      "amount": 0
    },
    "specifications": {
      "color": "Brown",
      "size": "Standard",
      "weight": null,
      "volume": null,
      "storage": null,
      "warranty": "1 year",
      "ram": null,
      "material": "Leather + steel",
      "other_specs": {}
    },
    "shipping": {
      "delivery_type": "doorstep",
      "method": "Bike Messenger",
      "station_location": null,
      "estimated_delivery_days": 1,
      "estimated_ready_in_hours": 2,
      "shipping_fee": 1000
    },
    "logistics": {
      "requires_heavy_transport": false,
      "truck_type": null,
      "offloading_required": false
    },
    "tax": {
      "vat_percentage": 7.5,
      "vat_amount": 5850
    },
    "final_total": 84850,
    "metadata": {
      "notes": "Comes in a gift box",
      "delivery_instructions": "Call before delivery",
      "gift_wrapping": true
    }
  },
  "category": "Accessories",
  "price": 78000,
  "unit": "piece",
  "quantity": 60,
  "available": true,
  "low_stock_threshold": 10
}
```

---

# ✅ **10. Pack of Printer Paper**

```json
{
  "product_id": null,
  "supplier_id": "34f1162c-5583-4adf-8880-02e35a403b85",
  "name": "A4 Premium Printer Paper (500 Sheets)",
  "description": {
    "name": "A4 Premium Printer Paper",
    "unit_price": 3200,
    "quantity": 500,
    "description": "High-quality A4 printing sheets for office use.",
    "currency": "NGN",
    "category": "Office Supplies",
    "sku": "PP-A4-500",
    "subtotal": 3200,
    "discount": {
      "type": "bulk",
      "amount": 300
    },
    "specifications": {
      "color": "White",
      "size": "A4",
      "weight": "80gsm",
      "volume": null,
      "storage": "Keep dry",
      "warranty": null,
      "ram": null,
      "material": "Paper",
      "other_specs": {}
    },
    "shipping": {
      "delivery_type": "pickup_station",
      "method": "Station Pickup",
      "station_location": "Yaba Station",
      "estimated_delivery_days": 1,
      "estimated_ready_in_hours": 5,
      "shipping_fee": 700
    },
    "logistics": {
      "requires_heavy_transport": false,
      "truck_type": null,
      "offloading_required": false
    },
    "tax": {
      "vat_percentage": 7.5,
      "vat_amount": 240
    },
    "final_total": 3140,
    "metadata": {
      "notes": null,
      "delivery_instructions": null,
      "gift_wrapping": false
    }
  },
  "category": "Office Supplies",
  "price": 3200,
  "unit": "pack",
  "quantity": 200,
  "available": true,
  "low_stock_threshold": 20
}
```


