//! # MacroDB
//!
//! This crate exports a macro that can be used to turn an appropriate Rust struct into an
//! in-memory database that guarantees consistency and supports indices. The Rust struct needs
//! to be written by hand, the macro only generates a safe interface for using the in-memory
//! database.
//!
//! ## Example
//!
//! Here is a full example of building a user database. An error type is defined, that is used
//! by the database methods to return information failures. A struct for the record type that is
//! to be stored is defined (the User struct). Finally, a struct for the database is defined
//! that contains two indices. The [table][macro@table] macro generates methods for inserting, updating
//! and deleting rows.
//!
//! ```rust
//! use std::collections::{BTreeMap, BTreeSet};
//! use macrodb::table;
//!
//! /// Error type for database interactions.
//! pub enum Error {
//!     UserIdExists,
//!     UserEmailExists,
//!     UserNotFound,
//! }
//!
//! type UserId = u64;
//!
//! /// Record type for a user (a row in the database).
//! #[derive(Clone)]
//! pub struct User {
//!     id: UserId,
//!     name: String,
//!     email: String,
//! }
//!
//! /// Database definition.
//! pub struct Database {
//!     /// Users table.
//!     users: BTreeMap<UserId, User>,
//!     /// Unique index of users by email.
//!     user_by_email: BTreeMap<String, UserId>,
//! }
//!
//! // The table macro will autogenerate the users_insert(), users_update() and users_delete()
//! // methods.
//! impl Database {
//!     table!(
//!         users: User,
//!         id: UserId,
//!         missing Error => Error::UserNotFound,
//!         primary users id => Error::UserIdExists,
//!         unique user_by_email email => Error::UserEmailExists
//!     );
//! }
//! ```
//!
//! See the documentation on [table](macro@table) for more information.
#![macro_use]

/// Re-expport of paste, which is used internally.
pub use paste::paste;

#[cfg(test)]
mod tests;

#[doc(hidden)]
#[macro_export]
macro_rules! table_next_id {
    ($table:ident: $type:ty) => {
        $crate::paste! {
            pub fn [<$table _next_id>](&self) -> $type {
                self.$table
                    .keys()
                    .max()
                    .map(|key| *key + 1)
                    .unwrap_or_default()
            }
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! table_insert {
    ($table:ident: $type:ty, $pk:ident, $errty:ty) => {
        $crate::paste! {
            pub fn [<$table _insert>](&mut self, data: $type) -> Result<(), $errty> {
                self.[<$table _insert_check>](&data)?;
                self.[<$table _insert_indices>](&data);
                self.$table.insert(data.$pk, data);
                Ok(())
            }
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! table_insert_check {
    ($self:expr, foreign, $table:ident, $expr:expr, $err:expr) => {
        if $self.$table.get(&$expr).is_none() {
            return Err($err);
        }
    };
    ($self:expr, unique, $table:ident, $expr:expr, $err:expr) => {
        if $self.$table.get(&$expr).is_some() {
            return Err($err);
        }
    };
    ($self:expr, primary, $table:ident, $expr:expr, $err:expr) => {
        if $self.$table.get(&$expr).is_some() {
            return Err($err);
        }
    };
    ($self:expr, reverse, $table:ident, $expr:expr, $err:expr) => {};
    ($self:expr, index, $table:ident, $expr:expr, $err:expr) => {};
}

#[doc(hidden)]
#[macro_export]
macro_rules! table_insert_checks {
    ($table:ident: $type:ty, $errty:ty, $($name:ident $ctable:ident $val:ident => $err:expr),*) => {
        $crate::paste! {
            fn [<$table _insert_check>](&mut self, data: &$type) -> Result<(), $errty> {
                $($crate::table_insert_check!(self, $name, $ctable, data.$val, $err);)*
                Ok(())
            }
        }
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! table_delete {
    ($table:ident: $type:ty, $pk:ty, $errty:ty) => {
        $crate::paste! {
            pub fn [<$table _delete>](&mut self, id: $pk) -> Result<$type, $errty> {
                let data = self.[<$table _delete_check>](id)?;
                self.[<$table _delete_indices>](&data);
                self.$table.remove(&id);
                Ok(data)
            }
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! table_insert_index {
    ($self:expr, $pk:expr, unique, $name:ident, $prop:expr) => {
        if $self.$name.insert($prop.clone(), $pk).is_some() {
            panic!(concat!(stringify!($name), " index entry already existsted"));
        }
    };
    ($self:expr, $pk:expr, reverse, $name:ident, $prop:expr) => {};
    ($self:expr, $pk:expr, index, $name:ident, $prop:expr) => {
        if !$self.$name.entry($prop.clone()).or_default().insert($pk) {
            panic!(concat!(stringify!($name), " index already had new user"));
        }
    };
    ($self:expr, $pk:expr, primary, $name:ident, $prop:expr) => {};
    ($self:expr, $pk:expr, foreign, $name:ident, $prop:expr) => {};
}

#[doc(hidden)]
#[macro_export]
macro_rules! table_insert_indices {
    ($table:ident: $type:ty, $pk:ident, $($itype:ident $name:ident $prop:ident => $err:expr),*) => {
        $crate::paste! {
            fn [<$table _insert_indices>](&mut self, data: &$type) {
                $($crate::table_insert_index!(self, data.$pk, $itype, $name, data.$prop);)*
            }
        }
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! table_delete_index {
    ($self:expr, $pk:expr, unique, $name:ident, $prop:expr) => {
        match $self.$name.remove(&$prop) {
            None => panic!(concat!(stringify!($name), " unique index missing item")),
            Some(value) if value != $pk => {
                panic!(concat!(stringify!($name), " unique index had wrong key"))
            }
            _ => {}
        }
    };
    ($self:expr, $pk:expr, reverse, $name:ident, $prop:expr) => {
        match $self.$name.remove(&$pk) {
            Some(value) if !value.is_empty() => {
                panic!(concat!(stringify!($name, " reverse index not empty")))
            }
            _ => {}
        }
    };
    ($self:expr, $pk:expr, index, $name:ident, $prop:expr) => {
        let values = $self
            .$name
            .get_mut(&$prop)
            .expect(concat!(stringify!($name), " index missing"));
        if !values.remove(&$pk) {
            panic!(concat!(stringify!($name), " index already had new user"));
        }
        if values.is_empty() {
            $self.$name.remove(&$prop);
        }
    };
    ($self:expr, $pk:expr, primary, $name:ident, $prop:expr) => {};
    ($self:expr, $pk:expr, foreign, $name:ident, $prop:expr) => {};
}

#[doc(hidden)]
#[macro_export]
macro_rules! table_delete_indices {
    ($table:ident: $type:ty, $pk:ident, $($itype:ident $name:ident $prop:ident => $err:expr),*) => {
        $crate::paste! {
            fn [<$table _delete_indices>](&mut self, data: &$type) {
                $($crate::table_delete_index!(self, data.$pk, $itype, $name, data.$prop);)*
            }
        }
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! table_update_index {
    ($self:expr, $pk:expr, index, $name:ident, $old:expr, $new:expr) => {
        if $old != $new {
            $crate::table_delete_index!($self, $pk, index, $name, $old);
            $crate::table_insert_index!($self, $pk, index, $name, $new);
        }
    };
    ($self:expr, $pk:expr, unique, $name:ident, $old:expr, $new:expr) => {
        if $old != $new {
            $crate::table_delete_index!($self, $pk, unique, $name, $old);
            $crate::table_insert_index!($self, $pk, unique, $name, $new);
        }
    };
    ($self:expr, $pk:expr, reverse, $name:ident, $old:expr, $new:expr) => {
        if $old != $new {
            $crate::table_delete_index!($self, $pk, reverse, $name, $old);
            $crate::table_insert_index!($self, $pk, reverse, $name, $new);
        }
    };
    ($self:expr, $pk:expr, foreign, $name:ident, $old:expr, $new:expr) => {};
    ($self:expr, $pk:expr, primary, $name:ident, $old:expr, $new:expr) => {};
}

#[doc(hidden)]
#[macro_export]
macro_rules! table_update_indices {
    ($table:ident: $type:ty, $pk:ident, $($itype:ident $name:ident $prop:ident => $err:expr),*) => {
        $crate::paste! {
            fn [<$table _update_indices>](&mut self, old: &$type, new: &$type) {
                $($crate::table_update_index!(self, old.$pk, $itype, $name, old.$prop, new.$prop);)*
            }
        }
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! table_update_check {
    ($self:expr, $pk:expr, unique, $name:ident, $old:expr, $new:expr, $err:expr) => {
        if $old != $new {
            $crate::table_insert_check!($self, unique, $name, $new, $err);
        }
    };
    ($self:expr, $pk:expr, foreign, $name:ident, $old:expr, $new:expr, $err:expr) => {
        if $old != $new {
            $crate::table_insert_check!($self, foreign, $name, $new, $err);
        }
    };
    ($self:expr, $pk:expr, primary, $name:ident, $old:expr, $new:expr, $err:expr) => {};
    ($self:expr, $pk:expr, reverse, $name:ident, $old:expr, $new:expr, $err:expr) => {};
    ($self:expr, $pk:expr, index, $name:ident, $old:expr, $new:expr, $err:expr) => {};
}

#[doc(hidden)]
#[macro_export]
macro_rules! table_update_checks {
    ($table:ident: $type:ty, $pk:ident, $errty:ty, $($itype:ident $name:ident $prop:ident => $err:expr),*) => {
        $crate::paste! {
            fn [<$table _update_check>](&mut self, old: &$type, new: &$type) -> Result<(), $errty> {
                $($crate::table_update_check!(self, old.$pk, $itype, $name, old.$prop, new.$prop, $err);)*
                Ok(())
            }
        }
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! table_delete_check {
    ($self:expr, $pk:expr, unique, $name:ident, $prop:expr, $err:expr) => {};
    ($self:expr, $pk:expr, foreign, $name:ident, $prop:expr, $err:expr) => {};
    ($self:expr, $pk:expr, primary, $name:ident, $prop:expr, $err:expr) => {};
    ($self:expr, $pk:expr, reverse, $name:ident, $prop:expr, $err:expr) => {
        match $self.$name.get(&$pk) {
            None => {}
            Some(items) if items.is_empty() => {
                panic!(concat!(stringify!($name), " has empty index"))
            }
            Some(_items) => return Err($err),
        }
    };
    ($self:expr, $pk:expr, index, $name:ident, $prop:expr, $err:expr) => {};
}

#[doc(hidden)]
#[macro_export]
macro_rules! table_delete_checks {
    ($table:ident: $type:ty, $pk:ident: $pkty:ty, $errty:ty, $missing:expr, $($itype:ident $name:ident $prop:ident => $err:expr),*) => {
        $crate::paste! {
            fn [<$table _delete_check>](&mut self, id: $pkty) -> Result<$type, $errty> {
                let row = match self.$table.get(&id) {
                    Some(row) => row.clone(),
                    None => return Err($missing),
                };

                $($crate::table_delete_check!(self, row.$pk, $itype, $name, row.$prop, $err);)*

                Ok(row)
            }
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! table_indices {
    ($table:ident: $type:ty, $pk:ident: $pkty:ty, $errty:ty, $error:expr, $($itype:ident $name:ident $prop:ident => $err:expr),*) => {
        $crate::table_insert_checks!($table: $type, $errty, $($itype $name $prop => $err),*);
        $crate::table_update_checks!($table: $type, $pk, $errty, $($itype $name $prop => $err),*);
        $crate::table_delete_checks!($table: $type, $pk: $pkty, $errty, $error, $($itype $name $prop => $err),*);
        $crate::table_insert_indices!($table: $type, $pk, $($itype $name $prop => $err),*);
        $crate::table_delete_indices!($table: $type, $pk, $($itype $name $prop => $err),*);
        $crate::table_update_indices!($table: $type, $pk, $($itype $name $prop => $err),*);
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! table_update {
    ($table:ident: $type:ty, $pk:ident => $err:expr, $errty:ty) => {
        $crate::paste! {
            pub fn [<$table _update>](&mut self, new: $type) -> Result<$type, $errty> {
                let old = match self.$table.get(&new.$pk) {
                    Some(value) => value.clone(),
                    None => return Err($err),
                };
                self.[<$table _update_check>](&old, &new)?;
                self.[<$table _update_indices>](&old, &new);
                self.$table.insert(new.$pk, new);
                Ok(old)
            }
        }
    };
}

/// # Table Macro
///
/// Generate database methods (insert, update and delete) for a table.
/// This macro takes a definition of the database table and indices and generates helper
/// methods used to safely insert, update and delete rows in the database.
///
/// ## Usage
///
/// In order to use the macro, you need an error type for your database,
/// a row type per table (for example, a User struct for your users table), and a struct
/// for the database itself.
///
/// ### Error Type
///
/// The error type is usually a simple enum. For every table, you need an error case for when a
/// row is missing, as well as when it exists. Additionally, you need one error type per unique
/// index.
///
/// Here is an example of the what the error type might look like:
///
/// ```rust
/// pub enum Error {
///     UserIdExists,
///     UserEmailExists,
///     UserNotFound,
///     GroupNameExists,
///     GroupIdExists,
///     GroupNotFound,
///     GroupNotEmpty,
/// }
/// ```
///
/// ### Row Types
///
/// For every table, you need a row type. This can just be a regular Rust struct. It needs to
/// derive [Clone][], and it needs to have some kind of primary key. The primary key uniquely
/// identifies the row for update and deletion operations. Usually, an integer type is
/// recommended here. For visual clarity, you can create a type alias for this field. We will
/// refer to the row type as `RowType` and to the type of the primary key as `RowId`.
///
/// Here is an example of what *User* and *Group* row types might look like.
///
/// ```rust
/// type UserId = u64;
///
/// #[derive(Clone)]
/// pub struct User {
///     id: UserId,
///     name: String,
///     email: String,
///     group: GroupId,
/// }
///
/// type GroupId = u64;
///
/// #[derive(Clone)]
/// pub struct Group {
///     id: GroupId,
///     name: String,
/// }
/// ```
///
///
/// ### Database Struct
///
/// The database struct must contain one map per table and one per index.
/// The table maps must be of the shape `MapType<RowPrimaryKey, RowType>`. The type of map that is used does not matter, although common choices are [BTreeMap][std::collections::BTreeMap]
/// and [HashMap][std::collections::HashMap].
/// For every unique index, a map of the shape `MapType<IndexType,
/// RowId>` must be added.
/// For every index, a map of the shape `MapType<IndexType, Set<RowId>>`
/// needs to be added.
/// The set type that is used
/// does not matter, although common choices are [BTreeSet][std::collections::BTreeSet]
/// and [HashSet][std::collections::HashSet].
/// The *IndexType* is the type of the field of the index. For instance, if
/// the index is on a field of type [String][], that is the IndexType.
///
/// | Kind | Type |
/// | --- | --- |
/// | Table | `MapType<RowId, RowType>` |
/// | Index | `MapType<IndexType, Set<RowId>>` |
/// | Unique index | `MapType<IndexType, RowId>` |
///
/// For example, to define a database struct with two tables (*users* and *groups*),
/// two unique indices (*user_by_email* and *group_by_name*), and one regular index
/// (*users_by_group*), this struct definition could be used:
///
/// ```rust
/// # type UserId = u64;
/// # type GroupId = u64;
/// # struct User;
/// # struct Group;
/// use std::collections::{BTreeMap, HashMap, BTreeSet, HashSet};
///
/// pub struct Database {
///     /// Users table
///     users: BTreeMap<UserId, User>,
///     /// User by email unique index
///     user_by_email: HashMap<String, UserId>,
///
///     /// Groups table
///     groups: HashMap<GroupId, Group>,
///     /// Users by group index
///     users_by_group: BTreeMap<GroupId, BTreeSet<UserId>>,
///     /// Group by name unique index
///     group_by_name: BTreeMap<String, GroupId>,
/// }
/// ```
///
/// ## Syntax
///
/// The basic syntax of the macro looks like this:
///
/// ```rust,ignore
/// table!(
///     $table_name: RowType,
///     $id_field: RowId,
///     missing ErrorType => $missing_error,
///     primary $table_name $id_field => $exists_error,
///     <indices...>
/// );
/// ```
///
/// Here is an overview of what the various placeholders mean:
///
/// | Placeholder | Example | Explanation |
/// | --- | --- | --- |
/// | `$table_name` | `users` | Name of the table map in the database struct |
/// | `RowType` | `User` | Name of the data type of the rows |
/// | `RowId` | `UserId` | Name of the type of the primary keys for the rows |
/// | `$id_field` | `id` | Name of the struct field of the Row type that contains the primary key |
/// | `ErrorType` | `Error` | Name of the error type (enum) |
/// | `$missing_error` | `Error::UserNotFound` | Error to throw when trying to delete a row that does not exists |
/// | `$exists_error` | `Error::UserIdExists` | Error to throw when trying to insert a row that already exists |
/// | `<indices...>` | | Definitions for the indices (explained in next section) |
///
/// The macro also needs to be told of the various indices. The syntax for indices looks like
/// this:
///
/// ```text
/// $type $map $field => $error
/// ```
///
/// Here, `$type` refers to the type of index (can be `index`, `unique`, `foreign` or
/// `reverse`). `$map` is the name of the map field in the database struct that represents this
/// index. `$field` is the name of the field of the `RowType` that this index is on. Finally,
/// `$error` is the error that is thrown when this index is violated. Here is an overview of the
/// index types:
///
/// | Type | Example | Explanation |
/// | --- | --- | --- |
/// | Index | `index users_by_group group => ()` | Defines a simple index to look up rows based on their group. Does not need an error. |
/// | Foreign | `foreign groups group => Error::GroupNotFound` | Defines a foreign key constraint which enforces that the `group` field point to an existing row in the `groups` table. |
/// | Unique | `unique user_by_email email => Error::UserEmailExists` | Defines a unique index which uses the `user_by_email` map and enforces that no two users share the same email. |
/// | Reverse | `reverse users_by_group id => Error::GroupHasUsers` | Declares a reverse dependency (on an index by another table) that prevents a group row being deleted if there are still users with that group. |
///
/// The result of this is that the macro generates insertion, update and deletion methods for
/// every table. It uses the table map name as the prefix for those methods. For example,
/// calling it on a table with the name *users* results in three methods being generated:
///
/// ```rust,ignore
/// impl Database {
///     /// Insert a User into the database, or return an error.
///     pub fn users_insert(row: User) -> Result<(), Error>;
///
///     /// Update a User row (identified by the primary key), returning the old row, or return
///     /// an error.
///     pub fn users_update(row: User) -> Result<User, Error>;
///
///     /// Delete a User row (identified by the primary key), returning the row, or return an
///     /// error.
///     pub fn users_delete(id: UserId) -> Result<User, Error>;
/// }
/// ```
///
/// ## Example
///
/// Here is an example invocation of the macro on the Database struct with two tables (*users* and
/// *groups*), including indices and foreign key constraints:
///
/// ```rust
/// # use std::collections::{BTreeMap, HashMap, BTreeSet, HashSet};
/// use macrodb::table;
/// # pub enum Error {
/// #     UserIdExists,
/// #     UserEmailExists,
/// #     UserNotFound,
/// #     GroupNameExists,
/// #     GroupIdExists,
/// #     GroupNotFound,
/// #     GroupNotEmpty,
/// # }
/// # type UserId = u64;
/// # #[derive(Clone)]
/// # pub struct User {
/// #     id: UserId,
/// #     name: String,
/// #     email: String,
/// #     group: GroupId,
/// # }
/// # type GroupId = u64;
/// # #[derive(Clone)]
/// # pub struct Group {
/// #     id: GroupId,
/// #     name: String,
/// # }
/// # pub struct Database {
/// #     users: BTreeMap<UserId, User>,
/// #     user_by_email: HashMap<String, UserId>,
/// #     groups: HashMap<GroupId, Group>,
/// #     users_by_group: BTreeMap<GroupId, BTreeSet<UserId>>,
/// #     group_by_name: BTreeMap<String, GroupId>,
/// # }
/// impl Database {
///     table!(
///         users: User,
///         id: UserId,
///         missing Error => Error::UserNotFound,
///         primary users id => Error::UserIdExists,
///         foreign groups group => Error::GroupNotFound,
///         index users_by_group group => (),
///         unique user_by_email email => Error::UserEmailExists
///     );
///     table!(
///         groups: Group,
///         id: GroupId,
///         missing Error => Error::GroupNotFound,
///         primary users id => Error::GroupIdExists,
///         reverse users_by_group id => Error::GroupNotEmpty,
///         unique group_by_name name => Error::GroupNameExists
///     );
/// }
/// ```
#[macro_export]
macro_rules! table {
    ($table:ident: $type:ty, $pk:ident: $pkty:ty, missing $errty:ty => $missing:expr, $($itype:ident $name:ident $prop:ident => $err:expr),*) => {
        $crate::table_next_id!($table: $pkty);
        $crate::table_indices!($table: $type, $pk: $pkty, $errty, $missing, $($itype $name $prop => $err),*);
        $crate::table_delete!($table: $type, $pkty, $errty);
        $crate::table_insert!($table: $type, $pk, $errty);
        $crate::table_update!($table: $type, $pk => $missing, $errty);
    }
}
