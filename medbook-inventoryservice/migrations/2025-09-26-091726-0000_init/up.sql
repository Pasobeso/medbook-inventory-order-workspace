-- Your SQL goes here

CREATE TABLE "inventory" (
  "product_id" integer PRIMARY KEY,
  "quantity" integer NOT NULL DEFAULT 0
);

CREATE TABLE "product" (
  "id" serial PRIMARY KEY,
  "th_name" text NOT NULL DEFAULT 'ชื่อผลิตภัณฑ์',
  "en_name" text NOT NULL DEFAULT 'Product Name'
);

ALTER TABLE "inventory" ADD FOREIGN KEY ("product_id") REFERENCES "product" ("id") ON DELETE CASCADE;

-- Seed Products
INSERT INTO product (id, th_name, en_name) VALUES
    (1, 'พาราเซตามอล', 'Paracetamol'),
    (2, 'ไอบูโพรเฟน', 'Ibuprofen'),
    (3, 'แอสไพริน', 'Aspirin'),
    (4, 'คลอร์เฟนิรามีน', 'Chlorpheniramine'),
    (5, 'ลอราทาดีน', 'Loratadine'),
    (6, 'ซิโปรฟลอกซาซิน', 'Ciprofloxacin'),
    (7, 'อะม็อกซิซิลลิน', 'Amoxicillin'),
    (8, 'โซเดียมไบคาร์บอเนต', 'Sodium Bicarbonate'),
    (9, 'โอเมพราโซล', 'Omeprazole'),
    (10, 'เกลือแร่', 'Oral Rehydration Salts');
