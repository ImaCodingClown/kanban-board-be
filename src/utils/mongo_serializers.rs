use mongodb::bson::oid::ObjectId;

pub fn opt_oid_to_str<S>(
    oid: &Option<ObjectId>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match oid {
        Some(oid) => serializer.serialize_some(&oid.to_hex()),
        None => serializer.serialize_none(),
    }
}
