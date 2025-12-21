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
        const FNV_OFFSET: u64 = 0xcbf29ce484222325;
        let mut hash = FNV_OFFSET;

        if self.include_tags {
            let tag_count: usize = self
                .error
                .context
                .iter()
                .filter_map(|ctx| match ctx {
                    ErrorContext::Group(g) => Some(g.tags.len()),
                    _ => None,
                })
                .sum();

            let mut tags = crate::types::alloc_type::Vec::with_capacity(tag_count);
            for ctx in &self.error.context {
                if let ErrorContext::Group(g) = ctx {
                    tags.extend_from_slice(&g.tags);
                }
            }
            tags.sort_unstable();

            for tag in tags {
                hash_bytes(&mut hash, b"tag:");
                hash_bytes(&mut hash, tag.as_bytes());
            }
        }

        if self.include_code {
            if let Some(code) = self.error.error_code {
                hash_bytes(&mut hash, b"code:");
                hash_bytes(&mut hash, &code.to_le_bytes());
            }
        }

        if self.include_message {
            hash_bytes(&mut hash, b"msg:");
            let mut hasher = DisplayHasher::new(&mut hash);
            let _ = write!(hasher, "{}", self.error.core_error);
        }

        if self.include_metadata {
            let meta_count: usize = self
                .error
                .context
                .iter()
                .filter_map(|ctx| match ctx {
                    ErrorContext::Group(g) => Some(g.metadata.len()),
                    _ => None,
                })
                .sum();

            let mut metadata = crate::types::alloc_type::Vec::with_capacity(meta_count);
            let all_metadata = self
                .error
                .context
                .iter()
                .filter_map(|ctx| match ctx {
                    ErrorContext::Group(g) => Some(g.metadata.iter()),
                    _ => None,
                })
                .flatten();

            for (k, v) in all_metadata {
                let key_str: &str = k.as_ref();
                let included = self
                    .include_keys
                    .map_or(true, |keys| keys.contains(&key_str));
                let excluded = self
                    .exclude_keys
                    .is_some_and(|keys| keys.contains(&key_str));

                if included && !excluded {
                    metadata.push((k, v));
                }
            }
            metadata.sort_unstable_by(|a, b| a.0.cmp(b.0));

            for (key, value) in metadata {
                hash_bytes(&mut hash, b"meta:");
                hash_bytes(&mut hash, key.as_bytes());
                hash_bytes(&mut hash, b"=");
                hash_bytes(&mut hash, value.as_bytes());
            }
        }

        hash
    }

    /// Computes the fingerprint and returns it as a hex string.
    #[must_use]
    pub fn compute_hex(&self) -> String
    where
        E: Display,
    {
        let mut result = String::with_capacity(16);
        let fp = self.compute();
        let _ = write!(result, "{:016x}", fp);
        result
    }
}

/// FNV-1a prime constant for 64-bit hash.
const FNV_PRIME: u64 = 0x100000001b3;

#[inline(always)]
fn hash_bytes(hash: &mut u64, bytes: &[u8]) {
    for &byte in bytes {
        *hash ^= byte as u64;
        *hash = hash.wrapping_mul(FNV_PRIME);
    }
}

struct DisplayHasher<'a> {
    hash: &'a mut u64,
}

impl<'a> DisplayHasher<'a> {
    #[inline(always)]
    fn new(hash: &'a mut u64) -> Self {
        Self { hash }
    }
}

impl Write for DisplayHasher<'_> {
    #[inline]
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        hash_bytes(self.hash, s.as_bytes());
        Ok(())
    }
}
