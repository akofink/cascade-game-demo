//! When a window size change is allowed to reconfigure the swapchain.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SurfaceChange {
    Unchanged,
    /// Minimized or otherwise empty. Keep the last non-zero configuration.
    SkipEmpty,
    Reconfigure {
        width: u32,
        height: u32,
    },
}

pub fn surface_change(configured: (u32, u32), requested: (u32, u32)) -> SurfaceChange {
    if requested.0 == 0 || requested.1 == 0 {
        SurfaceChange::SkipEmpty
    } else if requested == configured {
        SurfaceChange::Unchanged
    } else {
        SurfaceChange::Reconfigure {
            width: requested.0,
            height: requested.1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_size_does_not_reconfigure() {
        assert_eq!(
            surface_change((1280, 720), (0, 0)),
            SurfaceChange::SkipEmpty
        );
        assert_eq!(
            surface_change((1280, 720), (0, 400)),
            SurfaceChange::SkipEmpty
        );
    }

    #[test]
    fn same_size_is_unchanged_and_a_new_size_reconfigures_once() {
        assert_eq!(
            surface_change((800, 600), (800, 600)),
            SurfaceChange::Unchanged
        );
        assert_eq!(
            surface_change((800, 600), (900, 600)),
            SurfaceChange::Reconfigure {
                width: 900,
                height: 600
            }
        );
    }
}
