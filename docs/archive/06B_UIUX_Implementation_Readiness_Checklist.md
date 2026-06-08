# 06B — UI/UX Implementation Readiness Checklist

Version: v1.1

## Implementation Readiness

Do not start full UI implementation until all items below are complete.

## Layout

```txt
[ ] Responsive tiers defined
[ ] 1280px layout works
[ ] 1440px layout works
[ ] 1728px layout works
[ ] Sidebar collapse behavior defined
[ ] Inspector collapse behavior defined
[ ] Filmstrip hide/position behavior defined
[ ] Toolbar overflow behavior defined
```

## Component Usage

```txt
[ ] Global app frame implemented first
[ ] Component gallery exists
[ ] SrToolbar exists
[ ] SrSidebar exists
[ ] SrInspector exists
[ ] SrThumbnailGrid exists
[ ] SrLoupeViewer exists
[ ] SrAdjustmentSlider exists
[ ] SrExportDialog exists
```

## States

```txt
[ ] Empty state
[ ] Loading state
[ ] Partial loading state
[ ] Error state
[ ] Missing file state
[ ] Unsupported RAW state
[ ] Permission denied state
[ ] Export failed state
[ ] AI model unavailable state
```

## Accessibility

```txt
[ ] Keyboard-only Library flow
[ ] Keyboard-only Develop flow
[ ] Keyboard-only Export flow
[ ] Focus order documented
[ ] Icon-only buttons have labels
[ ] Reduced motion supported
[ ] Focus ring visible
[ ] Target sizes acceptable
```

## Photo Editing UX

```txt
[ ] Slider row behavior finalized
[ ] Numeric input finalized
[ ] Double-click reset finalized
[ ] Option-drag fine adjust finalized
[ ] Shift-drag coarse adjust finalized
[ ] Histogram behavior finalized
[ ] Before/after behavior finalized
[ ] Mask overlay behavior finalized
```

## Performance

```txt
[ ] Thumbnail grid virtualized
[ ] Filmstrip virtualized
[ ] Import does not block browsing
[ ] Slider-to-preview loop tested
[ ] Large library layout tested
```

## Mockups

```txt
[ ] Welcome
[ ] Library empty
[ ] Library populated
[ ] Library loupe
[ ] Develop default
[ ] Develop mask active
[ ] Export dialog
[ ] Preferences appearance
[ ] Import progress
[ ] AI review
```

## Final Gate

```txt
[ ] Product Designer GO
[ ] Apple HIG GO
[ ] Photographer Workflow GO
[ ] Frontend Engineer GO
[ ] OSS Maintainer GO
```
