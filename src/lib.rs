#![macro_use]
pub use paste::paste;

#[cfg(test)]
mod tests;

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
            Some(items) if !items.is_empty() => return Err($err),
            Some(items) => {}
        }
    };
    ($self:expr, $pk:expr, index, $name:ident, $prop:expr, $err:expr) => {};
}

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
