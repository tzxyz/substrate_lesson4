use codec::{Decode, Encode};
use scale_info::TypeInfo;
use serde::{Deserialize, Deserializer};
use sp_core::ConstU32;
use sp_runtime::BoundedVec;

#[derive(Debug, Deserialize, Encode, Decode, Clone, PartialEq, Eq, TypeInfo)]
pub struct GithubUser {
	pub id: u32,

	#[serde(deserialize_with = "de_string_to_bounded_bytes")]
	pub name: BoundedVec<u8, ConstU32<32>>,

	#[serde(deserialize_with = "de_string_to_bounded_bytes")]
	pub created_at: BoundedVec<u8, ConstU32<32>>,

	#[serde(deserialize_with = "de_string_to_bounded_bytes")]
	pub updated_at: BoundedVec<u8, ConstU32<32>>,

	pub public_repos: u32,
}

pub fn de_string_to_bounded_bytes<'de, D>(de: D) -> Result<BoundedVec<u8, ConstU32<32>>, D::Error>
where
	D: Deserializer<'de>,
{
	let s: &str = Deserialize::deserialize(de)?;
	let vec = BoundedVec::<u8, ConstU32<32>>::try_from(s.as_bytes().to_vec());
	Ok(vec.map_err(|_| serde::de::Error::custom("BoundedVec error"))?)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn deserialize_should_be_works() {
		let s = r#"{"login":"tzxyz","id":10796262,"node_id":"MDQ6VXNlcjEwNzk2MjYy","avatar_url":"https://avatars.githubusercontent.com/u/10796262?v=4","gravatar_id":"","url":"https://api.github.com/users/tzxyz","html_url":"https://github.com/tzxyz","followers_url":"https://api.github.com/users/tzxyz/followers","following_url":"https://api.github.com/users/tzxyz/following{/other_user}","gists_url":"https://api.github.com/users/tzxyz/gists{/gist_id}","starred_url":"https://api.github.com/users/tzxyz/starred{/owner}{/repo}","subscriptions_url":"https://api.github.com/users/tzxyz/subscriptions","organizations_url":"https://api.github.com/users/tzxyz/orgs","repos_url":"https://api.github.com/users/tzxyz/repos","events_url":"https://api.github.com/users/tzxyz/events{/privacy}","received_events_url":"https://api.github.com/users/tzxyz/received_events","type":"User","site_admin":false,"name":"tzxyz","company":null,"blog":"","location":"Beijing, China","email":null,"hireable":null,"bio":null,"twitter_username":null,"public_repos":33,"public_gists":0,"followers":7,"following":14,"created_at":"2015-02-01T11:59:33Z","updated_at":"2024-01-17T13:23:11Z"}"#;
		let user = serde_json::from_str::<GithubUser>(s).unwrap();
		println!("{:#?}", user);
	}
}
