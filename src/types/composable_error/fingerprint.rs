use crate::types::alloc_type::String;
use crate::types::composable_error::ComposableError;
use crate::types::ErrorContext;
use core::fmt::{Display, Write};

/// Configuration builder for customizing fingerprint generation.
pub struct FingerprintConfig<'a, E> {
    pub(crate) error: &'a ComposableError<E>,
    pub(crate) include_tags: bool,
    pub(crate) include_code: bool,
    pub(crate) include_message: bool,
    pub(crate) include_metadata: bool,
    pub(crate) include_keys: Option<&'a [&'a str]>,
    pub(crate) exclude_keys: Option<&'a [&'a str]>,
}

impl<'a, E> FingerprintConfig<'a, E> {
    pub(crate) fn new(error: &'a ComposableError<E>) -> Self {
        Self {
            error,
            include_tags: true,
            include_code: true,
            include_message: true,
            include_metadata: false,
            include_keys: None,
            exclude_keys: None,
        }
    }

    /// Whether to include tags in the fingerprint (default: true).
    #[must_use]
    pub fn include_tags(mut self, include: bool) -> Self {
        self.include_tags = include;
        self
    }

    /// Whether to include the error code in the fingerprint (default: true).
    #[must_use]
    pub fn include_code(mut self, include: bool) -> Self {
        self.include_code = include;
        self
    }

    /// Whether to include the core error message in the fingerprint (default: true).
    #[must_use]
    pub fn include_message(mut self, include: bool) -> Self {
        self.include_message = include;
        self
    }

    /// Whether to include metadata in the fingerprint (default: false).
    #[must_use]
    pub fn include_metadata(mut self, include: bool) -> Self {
        self.include_metadata = include;
        self
    }

    /// Explicitly include only these metadata keys in the fingerprint.
    #[must_use]
    pub fn include_metadata_keys(mut self, keys: &'a [&'a str]) -> Self {
        self.include_metadata = true;
        self.include_keys = Some(keys);
        self
    }

    /// Exclude these metadata keys from the fingerprint.
    #[must_use]
    pub fn exclude_metadata_keys(mut self, keys: &'a [&'a str]) -> Self {
        self.include_metadata = true;
        self.exclude_keys = Some(keys);
        self
    }

    /// Computes the fingerprint using the configured options.
    #[must_use]
    pub fn compute(&self) -> u64
    where
        E: Display,
    {
        let mut hasher = FnvHasher::new();

        if self.include_tags {
            self.hash_tags(&mut hasher);
        }

        if self.include_code {
            self.hash_code(&mut hasher);
        }

        if self.include_message {
            self.hash_message(&mut hasher);
        }

        if self.include_metadata {
            self.hash_metadata(&mut hasher);
        }

        hasher.finish()
    }

    #[inline]
    fn hash_tags(&self, hasher: &mut FnvHasher) {
        let tag_count: usize = self
            .error
            .context
            .iter()
            .filter_map(|ctx| match ctx {
                ErrorContext::Group(g) => Some(g.tags.len()),
                _ => None,
            })
            .sum();

        if tag_count == 0 {
            return;
        }

        let mut tags = crate::types::alloc_type::Vec::with_capacity(tag_count);
        for ctx in &self.error.context {
            if let ErrorContext::Group(g) = ctx {
                tags.extend_from_slice(&g.tags);
            }
        }
        tags.sort_unstable();

        for tag in tags {
            hasher.write(b"tag:");
            hasher.write(tag.as_bytes());
        }
    }

    #[inline]
    fn hash_code(&self, hasher: &mut FnvHasher) {
        if let Some(code) = self.error.error_code {
            hasher.write(b"code:");
            hasher.write(&code.to_le_bytes());
        }
    }

    #[inline]
    fn hash_message(&self, hasher: &mut FnvHasher)
    where
        E: Display,
    {
        hasher.write(b"msg:");
        let _ = write!(hasher, "{}", self.error.core_error);
    }

    #[inline]
    fn hash_metadata(&self, hasher: &mut FnvHasher) {
        let meta_count: usize = self
            .error
            .context
            .iter()
            .filter_map(|ctx| match ctx {
                ErrorContext::Group(g) => Some(g.metadata.len()),
                _ => None,
            })
            .sum();

        if meta_count == 0 {
            return;
        }

        let mut metadata = crate::types::alloc_type::Vec::with_capacity(meta_count);

        for ctx in &self.error.context {
            if let ErrorContext::Group(g) = ctx {
                for (k, v) in &g.metadata {
                    if self.should_include_key(k.as_ref()) {
                        metadata.push((k, v));
                    }
                }
            }
        }

        metadata.sort_unstable_by(|a, b| a.0.cmp(b.0));

        for (key, value) in metadata {
            hasher.write(b"meta:");
            hasher.write(key.as_bytes());
            hasher.write(b"=");
            hasher.write(value.as_bytes());
        }
    }

    #[inline]
    fn should_include_key(&self, key: &str) -> bool {
        let included = self.include_keys.map_or(true, |keys| keys.contains(&key));
        let excluded = self.exclude_keys.is_some_and(|keys| keys.contains(&key));
        included && !excluded
    }

    /// Computes the fingerprint and returns it as a hex string.
    #[must_use]
    pub fn compute_hex(&self) -> String
    where
        E: Display,
    {
        let mut result = String::with_capacity(16);
        let fp = self.compute();
        let _ = write!(result, "{fp:016x}");
        result
    }
}

/// FNV-1a hasher for 64-bit hash computation.
struct FnvHasher {
    hash: u64,
}

impl FnvHasher {
    const OFFSET: u64 = 0xcbf29ce484222325;
    const PRIME: u64 = 0x100000001b3;

    #[inline]
    const fn new() -> Self {
        Self { hash: Self::OFFSET }
    }

    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            self.hash ^= u64::from(byte);
            self.hash = self.hash.wrapping_mul(Self::PRIME);
        }
    }

    #[inline]
    const fn finish(self) -> u64 {
        self.hash
    }
}

impl Write for FnvHasher {
    #[inline]
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write(s.as_bytes());
        Ok(())
    }
}
