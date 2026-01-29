# GPUI Layout Patterns and Fixes

This document serves as a Claude skill for diagnosing and fixing common GPUI layout issues.

## Flexbox Width Constraint Issues

### Symptoms

- Sibling elements unexpectedly shrink to minimum width when window is resized
- Layout breaks at specific pixel thresholds (often when text is about to wrap)
- Container backgrounds don't extend full width
- Collapsing one section "fixes" the layout of other sections
- Problems appear/disappear based on content length

### Root Cause

In GPUI's Taffy-based flexbox implementation, containers with `flex_1()` and scrolling behavior (`overflow_y_scroll()`) may not properly constrain their width. When nested content (especially text) triggers layout recalculation (e.g., at wrap boundaries), the lack of explicit width constraints causes layout instability that propagates to sibling elements.

The issue stems from flexbox's default `min-width: auto` behavior, which sets an element's minimum width to its content's intrinsic width. Without explicit constraints, this can cause unexpected width expansion or collapse during layout recalculation.

### The Fix

Add `w_full()` and `min_w_0()` to scrollable flex containers:

```rust
// BEFORE (problematic)
div()
    .flex_1()
    .overflow_y_scroll()
    .p_4()
    .child(content)

// AFTER (fixed)
div()
    .w_full()           // Explicitly take full parent width
    .min_w_0()          // Allow shrinking below content's intrinsic width
    .flex_1()
    .overflow_y_scroll()
    .p_4()
    .child(content)
```

### Why It Works

- **`w_full()`**: Sets width to 100% of parent, ensuring the container sizes based on its parent rather than its content
- **`min_w_0()`**: Overrides the default `min-width: auto` behavior, allowing the element to shrink below its content's intrinsic minimum width

### Best Practices for GPUI Layouts

1. **Scrollable containers**: Always add `w_full()` and `min_w_0()` to containers with `overflow_y_scroll()` or `overflow_x_scroll()`

2. **Flex children that should shrink**: Add `min_w_0()` to any flex child that contains text or nested flex containers that should be allowed to shrink

3. **Nested flex layouts**: When nesting flex containers, ensure each level has proper width constraints:
   ```rust
   div()
       .w_full()
       .flex()
       .flex_col()
       .child(
           div()
               .w_full()
               .min_w_0()
               .flex()
               .flex_row()
               .child(/* ... */)
       )
   ```

4. **Text containers in flex layouts**: Text that may wrap should be in a container with `min_w_0()`:
   ```rust
   div()
       .flex_1()
       .min_w_0()  // Allows text container to shrink
       .child("Long text that may need to wrap...")
   ```

5. **Fixed + flexible layouts**: When combining fixed-width and flexible elements:
   ```rust
   div()
       .w_full()
       .flex()
       .flex_row()
       .child(
           div()
               .w(px(200.0))
               .flex_shrink_0()  // Don't shrink the fixed-width element
               .child("Label")
       )
       .child(
           div()
               .flex_1()
               .min_w_0()  // Allow flexible element to shrink
               .child(flexible_content)
       )
   ```

### Debugging Layout Issues

When encountering unexpected layout behavior:

1. **Isolate the problem**: Collapse/hide sections to identify which element causes the issue
2. **Check the flex chain**: Trace from the problematic element up to the root, ensuring each flex container has proper width constraints
3. **Test at boundaries**: Resize the window slowly to find the exact threshold where layout breaks
4. **Add constraints incrementally**: Start by adding `w_full()` and `min_w_0()` to the nearest scrollable ancestor, then work outward if needed

### Common Patterns

#### Scrollable Content Area
```rust
div()
    .id("main-content")
    .w_full()
    .min_w_0()
    .flex_1()
    .overflow_y_scroll()
    .p_4()
    .child(content)
```

#### Centered Max-Width Container
```rust
div()
    .w_full()
    .min_w_0()
    .flex()
    .flex_col()
    .max_w(px(900.0))
    .mx_auto()
    .child(content)
```

#### Form Row with Label and Input
```rust
div()
    .w_full()
    .min_w_0()
    .flex()
    .flex_row()
    .gap_4()
    .child(
        div()
            .w(px(150.0))
            .flex_shrink_0()
            .child("Label")
    )
    .child(
        div()
            .flex_1()
            .min_w_0()
            .child(input_widget)
    )
```

---

*Document created: 2026-01-29*
*Based on debugging session for FilePicker/DirectoryPicker layout issues in ccf-gpui-widgets*
