/*
 * test/mod.rs
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

extern crate color_backtrace;
extern crate tempfile;

mod authors;
mod page;
mod password;
mod tags;
mod user;
mod wiki;

use self::prelude::*;
use std::env;
use tempfile::tempdir;

mod prelude {
    pub use super::run;
    pub use crate::prelude::*;
    pub use either::*;
}

pub fn run<F: FnOnce(&Server)>(f: F) {
    color_backtrace::install();

    let database_url = &env::var("DATABASE_URL").expect("No DATABASE_URL specified!");
    let temp_dir = tempdir().expect("Unable to create temp dir");
    let revisions_dir = temp_dir.path().into();

    let config = ServerConfig {
        database_url,
        revisions_dir,
        password_blacklist: None,
    };

    let server = Server::new(config).expect("Unable to create server");

    server.test_transaction(|| {
        f(&server);
        Ok(())
    });
}
