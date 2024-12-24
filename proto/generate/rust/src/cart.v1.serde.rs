// @generated
impl serde::Serialize for AddItemRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.id.is_empty() {
            len += 1;
        }
        if !self.tenant_id.is_empty() {
            len += 1;
        }
        if !self.item_id.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cart.v1.AddItemRequest", len)?;
        if !self.id.is_empty() {
            struct_ser.serialize_field("id", &self.id)?;
        }
        if !self.tenant_id.is_empty() {
            struct_ser.serialize_field("tenantId", &self.tenant_id)?;
        }
        if !self.item_id.is_empty() {
            struct_ser.serialize_field("itemId", &self.item_id)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for AddItemRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "id",
            "tenant_id",
            "tenantId",
            "item_id",
            "itemId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Id,
            TenantId,
            ItemId,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "id" => Ok(GeneratedField::Id),
                            "tenantId" | "tenant_id" => Ok(GeneratedField::TenantId),
                            "itemId" | "item_id" => Ok(GeneratedField::ItemId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = AddItemRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cart.v1.AddItemRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<AddItemRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut id__ = None;
                let mut tenant_id__ = None;
                let mut item_id__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Id => {
                            if id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("id"));
                            }
                            id__ = Some(map.next_value()?);
                        }
                        GeneratedField::TenantId => {
                            if tenant_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tenantId"));
                            }
                            tenant_id__ = Some(map.next_value()?);
                        }
                        GeneratedField::ItemId => {
                            if item_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("itemId"));
                            }
                            item_id__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(AddItemRequest {
                    id: id__.unwrap_or_default(),
                    tenant_id: tenant_id__.unwrap_or_default(),
                    item_id: item_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cart.v1.AddItemRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for AddItemResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("cart.v1.AddItemResponse", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for AddItemResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                            Err(serde::de::Error::unknown_field(value, FIELDS))
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = AddItemResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cart.v1.AddItemResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<AddItemResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(AddItemResponse {
                })
            }
        }
        deserializer.deserialize_struct("cart.v1.AddItemResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CartCreatedV1 {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("cart.v1.CartCreatedV1", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CartCreatedV1 {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                            Err(serde::de::Error::unknown_field(value, FIELDS))
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CartCreatedV1;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cart.v1.CartCreatedV1")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<CartCreatedV1, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(CartCreatedV1 {
                })
            }
        }
        deserializer.deserialize_struct("cart.v1.CartCreatedV1", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CreateRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("cart.v1.CreateRequest", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CreateRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                            Err(serde::de::Error::unknown_field(value, FIELDS))
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CreateRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cart.v1.CreateRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<CreateRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(CreateRequest {
                })
            }
        }
        deserializer.deserialize_struct("cart.v1.CreateRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CreateResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.id.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cart.v1.CreateResponse", len)?;
        if !self.id.is_empty() {
            struct_ser.serialize_field("id", &self.id)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CreateResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "id",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Id,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "id" => Ok(GeneratedField::Id),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CreateResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cart.v1.CreateResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<CreateResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut id__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Id => {
                            if id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("id"));
                            }
                            id__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(CreateResponse {
                    id: id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cart.v1.CreateResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Event {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.id != 0 {
            len += 1;
        }
        if !self.aggregate_id.is_empty() {
            len += 1;
        }
        if self.payload.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cart.v1.Event", len)?;
        if self.id != 0 {
            struct_ser.serialize_field("id", ToString::to_string(&self.id).as_str())?;
        }
        if !self.aggregate_id.is_empty() {
            struct_ser.serialize_field("aggregateId", &self.aggregate_id)?;
        }
        if let Some(v) = self.payload.as_ref() {
            match v {
                event::Payload::CartCreatedV1(v) => {
                    struct_ser.serialize_field("cartCreatedV1", v)?;
                }
                event::Payload::ItemAddedV1(v) => {
                    struct_ser.serialize_field("itemAddedV1", v)?;
                }
                event::Payload::ItemRemovedV1(v) => {
                    struct_ser.serialize_field("itemRemovedV1", v)?;
                }
                event::Payload::OrderPlacedV1(v) => {
                    struct_ser.serialize_field("orderPlacedV1", v)?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Event {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "id",
            "aggregate_id",
            "aggregateId",
            "cart_created_v1",
            "cartCreatedV1",
            "item_added_v1",
            "itemAddedV1",
            "item_removed_v1",
            "itemRemovedV1",
            "order_placed_v1",
            "orderPlacedV1",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Id,
            AggregateId,
            CartCreatedV1,
            ItemAddedV1,
            ItemRemovedV1,
            OrderPlacedV1,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "id" => Ok(GeneratedField::Id),
                            "aggregateId" | "aggregate_id" => Ok(GeneratedField::AggregateId),
                            "cartCreatedV1" | "cart_created_v1" => Ok(GeneratedField::CartCreatedV1),
                            "itemAddedV1" | "item_added_v1" => Ok(GeneratedField::ItemAddedV1),
                            "itemRemovedV1" | "item_removed_v1" => Ok(GeneratedField::ItemRemovedV1),
                            "orderPlacedV1" | "order_placed_v1" => Ok(GeneratedField::OrderPlacedV1),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Event;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cart.v1.Event")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Event, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut id__ = None;
                let mut aggregate_id__ = None;
                let mut payload__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Id => {
                            if id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("id"));
                            }
                            id__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::AggregateId => {
                            if aggregate_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("aggregateId"));
                            }
                            aggregate_id__ = Some(map.next_value()?);
                        }
                        GeneratedField::CartCreatedV1 => {
                            if payload__.is_some() {
                                return Err(serde::de::Error::duplicate_field("cartCreatedV1"));
                            }
                            payload__ = map.next_value::<::std::option::Option<_>>()?.map(event::Payload::CartCreatedV1)
;
                        }
                        GeneratedField::ItemAddedV1 => {
                            if payload__.is_some() {
                                return Err(serde::de::Error::duplicate_field("itemAddedV1"));
                            }
                            payload__ = map.next_value::<::std::option::Option<_>>()?.map(event::Payload::ItemAddedV1)
;
                        }
                        GeneratedField::ItemRemovedV1 => {
                            if payload__.is_some() {
                                return Err(serde::de::Error::duplicate_field("itemRemovedV1"));
                            }
                            payload__ = map.next_value::<::std::option::Option<_>>()?.map(event::Payload::ItemRemovedV1)
;
                        }
                        GeneratedField::OrderPlacedV1 => {
                            if payload__.is_some() {
                                return Err(serde::de::Error::duplicate_field("orderPlacedV1"));
                            }
                            payload__ = map.next_value::<::std::option::Option<_>>()?.map(event::Payload::OrderPlacedV1)
;
                        }
                    }
                }
                Ok(Event {
                    id: id__.unwrap_or_default(),
                    aggregate_id: aggregate_id__.unwrap_or_default(),
                    payload: payload__,
                })
            }
        }
        deserializer.deserialize_struct("cart.v1.Event", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GetRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.id.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cart.v1.GetRequest", len)?;
        if !self.id.is_empty() {
            struct_ser.serialize_field("id", &self.id)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GetRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "id",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Id,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "id" => Ok(GeneratedField::Id),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = GetRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cart.v1.GetRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<GetRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut id__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Id => {
                            if id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("id"));
                            }
                            id__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(GetRequest {
                    id: id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cart.v1.GetRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GetResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.id.is_empty() {
            len += 1;
        }
        if !self.items.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cart.v1.GetResponse", len)?;
        if !self.id.is_empty() {
            struct_ser.serialize_field("id", &self.id)?;
        }
        if !self.items.is_empty() {
            struct_ser.serialize_field("items", &self.items)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GetResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "id",
            "items",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Id,
            Items,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "id" => Ok(GeneratedField::Id),
                            "items" => Ok(GeneratedField::Items),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = GetResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cart.v1.GetResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<GetResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut id__ = None;
                let mut items__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Id => {
                            if id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("id"));
                            }
                            id__ = Some(map.next_value()?);
                        }
                        GeneratedField::Items => {
                            if items__.is_some() {
                                return Err(serde::de::Error::duplicate_field("items"));
                            }
                            items__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(GetResponse {
                    id: id__.unwrap_or_default(),
                    items: items__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cart.v1.GetResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for get_response::Item {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.tenant_id.is_empty() {
            len += 1;
        }
        if !self.item_id.is_empty() {
            len += 1;
        }
        if self.quantity != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cart.v1.GetResponse.Item", len)?;
        if !self.tenant_id.is_empty() {
            struct_ser.serialize_field("tenantId", &self.tenant_id)?;
        }
        if !self.item_id.is_empty() {
            struct_ser.serialize_field("itemId", &self.item_id)?;
        }
        if self.quantity != 0 {
            struct_ser.serialize_field("quantity", &self.quantity)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for get_response::Item {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "tenant_id",
            "tenantId",
            "item_id",
            "itemId",
            "quantity",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            TenantId,
            ItemId,
            Quantity,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "tenantId" | "tenant_id" => Ok(GeneratedField::TenantId),
                            "itemId" | "item_id" => Ok(GeneratedField::ItemId),
                            "quantity" => Ok(GeneratedField::Quantity),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = get_response::Item;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cart.v1.GetResponse.Item")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<get_response::Item, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut tenant_id__ = None;
                let mut item_id__ = None;
                let mut quantity__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::TenantId => {
                            if tenant_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tenantId"));
                            }
                            tenant_id__ = Some(map.next_value()?);
                        }
                        GeneratedField::ItemId => {
                            if item_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("itemId"));
                            }
                            item_id__ = Some(map.next_value()?);
                        }
                        GeneratedField::Quantity => {
                            if quantity__.is_some() {
                                return Err(serde::de::Error::duplicate_field("quantity"));
                            }
                            quantity__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(get_response::Item {
                    tenant_id: tenant_id__.unwrap_or_default(),
                    item_id: item_id__.unwrap_or_default(),
                    quantity: quantity__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cart.v1.GetResponse.Item", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ItemAddedV1 {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.tenant_id.is_empty() {
            len += 1;
        }
        if !self.item_id.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cart.v1.ItemAddedV1", len)?;
        if !self.tenant_id.is_empty() {
            struct_ser.serialize_field("tenantId", &self.tenant_id)?;
        }
        if !self.item_id.is_empty() {
            struct_ser.serialize_field("itemId", &self.item_id)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ItemAddedV1 {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "tenant_id",
            "tenantId",
            "item_id",
            "itemId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            TenantId,
            ItemId,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "tenantId" | "tenant_id" => Ok(GeneratedField::TenantId),
                            "itemId" | "item_id" => Ok(GeneratedField::ItemId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ItemAddedV1;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cart.v1.ItemAddedV1")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ItemAddedV1, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut tenant_id__ = None;
                let mut item_id__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::TenantId => {
                            if tenant_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tenantId"));
                            }
                            tenant_id__ = Some(map.next_value()?);
                        }
                        GeneratedField::ItemId => {
                            if item_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("itemId"));
                            }
                            item_id__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(ItemAddedV1 {
                    tenant_id: tenant_id__.unwrap_or_default(),
                    item_id: item_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cart.v1.ItemAddedV1", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ItemRemovedV1 {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.tenant_id.is_empty() {
            len += 1;
        }
        if !self.item_id.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cart.v1.ItemRemovedV1", len)?;
        if !self.tenant_id.is_empty() {
            struct_ser.serialize_field("tenantId", &self.tenant_id)?;
        }
        if !self.item_id.is_empty() {
            struct_ser.serialize_field("itemId", &self.item_id)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ItemRemovedV1 {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "tenant_id",
            "tenantId",
            "item_id",
            "itemId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            TenantId,
            ItemId,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "tenantId" | "tenant_id" => Ok(GeneratedField::TenantId),
                            "itemId" | "item_id" => Ok(GeneratedField::ItemId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ItemRemovedV1;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cart.v1.ItemRemovedV1")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ItemRemovedV1, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut tenant_id__ = None;
                let mut item_id__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::TenantId => {
                            if tenant_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tenantId"));
                            }
                            tenant_id__ = Some(map.next_value()?);
                        }
                        GeneratedField::ItemId => {
                            if item_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("itemId"));
                            }
                            item_id__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(ItemRemovedV1 {
                    tenant_id: tenant_id__.unwrap_or_default(),
                    item_id: item_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cart.v1.ItemRemovedV1", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for OrderPlacedV1 {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.items.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cart.v1.OrderPlacedV1", len)?;
        if !self.items.is_empty() {
            struct_ser.serialize_field("items", &self.items)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for OrderPlacedV1 {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "items",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Items,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "items" => Ok(GeneratedField::Items),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = OrderPlacedV1;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cart.v1.OrderPlacedV1")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<OrderPlacedV1, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut items__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Items => {
                            if items__.is_some() {
                                return Err(serde::de::Error::duplicate_field("items"));
                            }
                            items__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(OrderPlacedV1 {
                    items: items__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cart.v1.OrderPlacedV1", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for order_placed_v1::Item {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.tenant_id.is_empty() {
            len += 1;
        }
        if !self.item_id.is_empty() {
            len += 1;
        }
        if self.quantity != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cart.v1.OrderPlacedV1.Item", len)?;
        if !self.tenant_id.is_empty() {
            struct_ser.serialize_field("tenantId", &self.tenant_id)?;
        }
        if !self.item_id.is_empty() {
            struct_ser.serialize_field("itemId", &self.item_id)?;
        }
        if self.quantity != 0 {
            struct_ser.serialize_field("quantity", &self.quantity)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for order_placed_v1::Item {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "tenant_id",
            "tenantId",
            "item_id",
            "itemId",
            "quantity",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            TenantId,
            ItemId,
            Quantity,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "tenantId" | "tenant_id" => Ok(GeneratedField::TenantId),
                            "itemId" | "item_id" => Ok(GeneratedField::ItemId),
                            "quantity" => Ok(GeneratedField::Quantity),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = order_placed_v1::Item;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cart.v1.OrderPlacedV1.Item")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<order_placed_v1::Item, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut tenant_id__ = None;
                let mut item_id__ = None;
                let mut quantity__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::TenantId => {
                            if tenant_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tenantId"));
                            }
                            tenant_id__ = Some(map.next_value()?);
                        }
                        GeneratedField::ItemId => {
                            if item_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("itemId"));
                            }
                            item_id__ = Some(map.next_value()?);
                        }
                        GeneratedField::Quantity => {
                            if quantity__.is_some() {
                                return Err(serde::de::Error::duplicate_field("quantity"));
                            }
                            quantity__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(order_placed_v1::Item {
                    tenant_id: tenant_id__.unwrap_or_default(),
                    item_id: item_id__.unwrap_or_default(),
                    quantity: quantity__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cart.v1.OrderPlacedV1.Item", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for PlaceOrderRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.id.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cart.v1.PlaceOrderRequest", len)?;
        if !self.id.is_empty() {
            struct_ser.serialize_field("id", &self.id)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for PlaceOrderRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "id",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Id,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "id" => Ok(GeneratedField::Id),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = PlaceOrderRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cart.v1.PlaceOrderRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<PlaceOrderRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut id__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Id => {
                            if id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("id"));
                            }
                            id__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(PlaceOrderRequest {
                    id: id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cart.v1.PlaceOrderRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for PlaceOrderResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("cart.v1.PlaceOrderResponse", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for PlaceOrderResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                            Err(serde::de::Error::unknown_field(value, FIELDS))
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = PlaceOrderResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cart.v1.PlaceOrderResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<PlaceOrderResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(PlaceOrderResponse {
                })
            }
        }
        deserializer.deserialize_struct("cart.v1.PlaceOrderResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for RemoveItemRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.id.is_empty() {
            len += 1;
        }
        if !self.tenant_id.is_empty() {
            len += 1;
        }
        if !self.item_id.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cart.v1.RemoveItemRequest", len)?;
        if !self.id.is_empty() {
            struct_ser.serialize_field("id", &self.id)?;
        }
        if !self.tenant_id.is_empty() {
            struct_ser.serialize_field("tenantId", &self.tenant_id)?;
        }
        if !self.item_id.is_empty() {
            struct_ser.serialize_field("itemId", &self.item_id)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for RemoveItemRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "id",
            "tenant_id",
            "tenantId",
            "item_id",
            "itemId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Id,
            TenantId,
            ItemId,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "id" => Ok(GeneratedField::Id),
                            "tenantId" | "tenant_id" => Ok(GeneratedField::TenantId),
                            "itemId" | "item_id" => Ok(GeneratedField::ItemId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = RemoveItemRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cart.v1.RemoveItemRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<RemoveItemRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut id__ = None;
                let mut tenant_id__ = None;
                let mut item_id__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Id => {
                            if id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("id"));
                            }
                            id__ = Some(map.next_value()?);
                        }
                        GeneratedField::TenantId => {
                            if tenant_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tenantId"));
                            }
                            tenant_id__ = Some(map.next_value()?);
                        }
                        GeneratedField::ItemId => {
                            if item_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("itemId"));
                            }
                            item_id__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(RemoveItemRequest {
                    id: id__.unwrap_or_default(),
                    tenant_id: tenant_id__.unwrap_or_default(),
                    item_id: item_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cart.v1.RemoveItemRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for RemoveItemResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("cart.v1.RemoveItemResponse", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for RemoveItemResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                            Err(serde::de::Error::unknown_field(value, FIELDS))
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = RemoveItemResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cart.v1.RemoveItemResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<RemoveItemResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(RemoveItemResponse {
                })
            }
        }
        deserializer.deserialize_struct("cart.v1.RemoveItemResponse", FIELDS, GeneratedVisitor)
    }
}
