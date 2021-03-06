/*
 * session/service.rs
 *
 * deepwell - Database management and migrations service
 * Copyright (C) 2019 Ammon Smith
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program. If not, see <http://www.gnu.org/licenses/>.
 */

use super::NewSession;
use crate::schema::sessions;
use crate::service_prelude::*;
use crate::utils::rows_to_result;
use chrono::prelude::*;
use ipnetwork::IpNetwork;
use rand::{distributions::Alphanumeric, rngs::OsRng, Rng};
use std::iter;

const TOKEN_LENGTH: usize = 64;

// This implementation is extremely primitive -- it just stores a securely-generated
// random string as the token and then matches it when the user makes calls.
//
// In the future we will want distinct session objects which are separated by IP and
// can be invalidated separately.
//
// This also might want to be in-memory instead of persisted.

#[derive(Debug, Queryable)]
pub struct Session {
    user_id: UserId,
    token: String,
    ip_address: IpNetwork,
    created_at: DateTime<Utc>,
}

impl Session {
    #[inline]
    pub fn user_id(&self) -> UserId {
        self.user_id
    }

    #[inline]
    pub fn token(&self) -> &str {
        &self.token
    }

    #[inline]
    pub fn ip_address(&self) -> IpNetwork {
        self.ip_address
    }
}

pub struct SessionService {
    conn: Arc<PgConnection>,
}

impl SessionService {
    #[inline]
    pub fn new(conn: &Arc<PgConnection>) -> Self {
        let conn = Arc::clone(conn);
        SessionService { conn }
    }

    pub fn get_session(&self, user_id: UserId) -> Result<Option<Session>> {
        info!("Getting session information any for user ID {}", user_id);

        let id: i64 = user_id.into();
        let session = sessions::table
            .find(id)
            .first::<Session>(&*self.conn)
            .optional()?;

        Ok(session)
    }

    pub fn get_token(&self, user_id: UserId) -> Result<Option<String>> {
        debug!("Getting token (if any) for user ID {}", user_id);

        let id: i64 = user_id.into();
        let token = sessions::table
            .find(id)
            .select(sessions::dsl::token)
            .first::<String>(&*self.conn)
            .optional()?;

        Ok(token)
    }

    pub fn check_token(&self, user_id: UserId, token: &str) -> Result<()> {
        debug!("Checking token for user ID {}", user_id);

        let id: i64 = user_id.into();
        let result = sessions::table
            .find(id)
            .filter(sessions::dsl::token.eq(token))
            .select(sessions::dsl::user_id)
            .first::<UserId>(&*self.conn)
            .optional()?;

        match result {
            Some(_) => Ok(()),
            None => Err(Error::InvalidToken),
        }
    }

    pub fn create_token(&self, user_id: UserId, ip_address: IpNetwork) -> Result<String> {
        debug!("Creating token for user ID {}", user_id);

        let token = generate_token();
        let model = NewSession {
            user_id: user_id.into(),
            token: &token,
            ip_address,
        };

        diesel::insert_into(sessions::table)
            .values(&model)
            .execute(&*self.conn)?;

        Ok(token)
    }

    pub fn revoke_token(&self, user_id: UserId) -> Result<bool> {
        debug!("Revoking token for user ID {}", user_id);

        let id: i64 = user_id.into();
        let rows = diesel::delete(sessions::table)
            .filter(sessions::dsl::user_id.eq(id))
            .execute(&*self.conn)?;

        Ok(rows_to_result(rows))
    }
}

fn generate_token() -> String {
    iter::repeat(())
        .map(|_| OsRng.sample(Alphanumeric))
        .take(TOKEN_LENGTH)
        .collect()
}
