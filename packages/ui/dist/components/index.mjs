import { componentToneClass } from "./tone.mjs";
import { Badge, badgeClass } from "./badge.mjs";
import { ActivityStatus, ActivityStatusIndicator } from "./activity-status.mjs";
import { Kbd, KbdGroup, Skeleton, Spinner } from "./display.mjs";
import { DevServerErrorOverlay } from "./dev-server-error.mjs";
import { ComponentsProvider, componentsControlSize, componentsElevation, componentsThemeContract, useComponentsTheme } from "./theme.mjs";
import { Button } from "./button.mjs";
import { Alert, AlertActions, AlertDescription, AlertTitle, alertColors } from "./alert.mjs";
import { Dialog, DialogDescription, DialogFooter, DialogHeader, DialogScrollBody, DialogTitle } from "./dialog.mjs";
import { AlertDialog, AlertDialogAction, AlertDialogCancel, AlertDialogDescription, AlertDialogFooter, AlertDialogHeader, AlertDialogTitle } from "./alert-dialog.mjs";
import { AspectRatio, aspectRatioStyle } from "./aspect-ratio.mjs";
import { Attachment, AttachmentAction, AttachmentActions, AttachmentContent, AttachmentDescription, AttachmentGroup, AttachmentMedia, AttachmentTitle, attachmentClass, attachmentGroupClass, attachmentMediaClass } from "./attachment.mjs";
import { Avatar, AvatarGroup, AvatarGroupCount, avatarInitials } from "./avatar.mjs";
import { ButtonGroup, ButtonGroupSeparator, ButtonGroupText } from "./button-group.mjs";
import { Card, CardAction, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from "./card.mjs";
import { Carousel, CarouselContent, CarouselItem, CarouselNext, CarouselPrevious, normalizeCarouselIndex } from "./carousel.mjs";
import { ChartContainer, ChartEmpty, ChartLegend, useChartConfig } from "./chart.mjs";
import { CopyButton } from "./copy-button.mjs";
import { CodeBlock } from "./code-block.mjs";
import { moveMenuHighlight } from "./menu-state.mjs";
import { filterCommandItems, reconcileCommandHighlight } from "./command-state.mjs";
import { Input, PasswordInput, TextArea } from "./input.mjs";
import { Command, CommandList, createCommandListNavigation } from "./command.mjs";
import { Combobox } from "./combobox.mjs";
import { ContentState, ResourceBoundary } from "./content-state.mjs";
import { DropdownMenu } from "./dropdown-menu.mjs";
import { ContextMenu } from "./context-menu.mjs";
import { Table, TableBody, TableCaption, TableCell, TableFooter, TableHead, TableHeader, TableRow } from "./table.mjs";
import { DataTable, TableSortIndicator } from "./data-table.mjs";
import { Calendar, CalendarDate, DatePicker } from "./date-picker.mjs";
import { Accordion, AccordionContent, AccordionItem, AccordionTrigger, Collapsible, CollapsibleContent, CollapsibleTrigger, nextAccordionValue } from "./disclosure.mjs";
import { DiffViewer } from "./diff-viewer.mjs";
import { DirectionProvider, DirectionalRow, DirectionalText, useDirection } from "./direction.mjs";
import { Label } from "./label.mjs";
import { Field, FieldContent, FieldDescription, FieldError, FieldGroup, FieldLabel, FieldLegend, FieldSeparator, FieldSet, FieldTitle, InputGroup, InputGroupAddon, InputGroupButton, InputGroupInput, InputGroupText, InputGroupTextArea, LabeledField, fieldClass, fieldErrorLabel, inputGroupAddonClass, inputGroupClass, uniqueFieldErrors } from "./forms.mjs";
import { DirectoryPicker } from "./directory-picker.mjs";
import { Drawer, DrawerClose, DrawerDescription, DrawerFooter, DrawerHandle, DrawerHeader, DrawerTitle, drawerDragOffset, drawerShouldDismiss } from "./drawer.mjs";
import { DropZone, pointInLayoutRect } from "./drop-zone.mjs";
import { Empty, EmptyContent, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle, emptyClass, emptyMediaClass } from "./empty.mjs";
import { GroupBox, groupBoxContentClass } from "./group-box.mjs";
import { createDelayedOpenController } from "./delayed-open.mjs";
import { HoverCard } from "./hover-card.mjs";
import { IconFrame } from "./icon-frame.mjs";
import { ImageList } from "./image-list.mjs";
import { AnnotationLayer, ImageOverlayLayer, ImageViewport, clampAnnotationRegion, imageViewportTransform } from "./image-viewport.mjs";
import { InlineEdit } from "./inline-edit.mjs";
import { InputOTP, InputOTPGroup, InputOTPSeparator, InputOTPSlot, normalizeOtpValue } from "./input-otp.mjs";
import { Item, ItemActions, ItemContent, ItemDescription, ItemFooter, ItemGroup, ItemHeader, ItemMedia, ItemSeparator, ItemTitle, itemClass, itemMediaClass } from "./item.mjs";
import { AdaptiveSplitPane, AdaptiveSplitPaneDetail, AdaptiveSplitPaneMain, ResponsiveGrid, ResponsiveGridRemainder, SplitPane, SplitPaneAside, SplitPaneMain, responsiveGridColumnCount, responsiveGridRemainderCount, useResponsiveGrid } from "./layout.mjs";
import { Listbox } from "./listbox.mjs";
import { LabeledSeparator, Separator } from "./separator.mjs";
import { Markdown } from "./markdown.mjs";
import { Toolbar, ToolbarButton, ToolbarGroup, ToolbarSeparator, ToolbarToggle } from "./toolbar.mjs";
import { Menubar, MenubarMenu } from "./menubar.mjs";
import { Bubble, BubbleContent, BubbleGroup, BubbleReactions, Marker, MarkerContent, MarkerIcon, Message, MessageActions, MessageAvatar, MessageContent, MessageFooter, MessageGroup, MessageHeader, bubbleClass, bubbleContentClass, messageActionsClass, messageClass, messageContentClass } from "./message.mjs";
import { Tooltip, TooltipContent } from "./tooltip.mjs";
import { MessageScroller, MessageScrollerButton, MessageScrollerContent, MessageScrollerItem, MessageScrollerNavigator, MessageScrollerViewport, activeMessageAnchor, isMessageScrollNearEnd, messageScrollRange, messageScrollRevealDelta, messageScrollStartDelta, useMessageScroller } from "./message-scroller.mjs";
import { clampPage, createPaginationRange, normalizePageCount } from "./pagination-state.mjs";
import { Breadcrumb, BreadcrumbEllipsis, BreadcrumbItem, BreadcrumbLink, BreadcrumbList, BreadcrumbPage, BreadcrumbSeparator, Pagination, PaginationContent, PaginationEllipsis, PaginationItem, PaginationItems, PaginationLink, PaginationNext, PaginationPrevious } from "./navigation.mjs";
import { Popover, PopoverDescription, PopoverFooter, PopoverHeader, PopoverTitle } from "./popover.mjs";
import { NavigationMenu, NavigationMenuContent, NavigationMenuIndicator, NavigationMenuItem, NavigationMenuLink, NavigationMenuList, NavigationMenuTrigger, NavigationMenuViewport, navigationMenuTriggerClass } from "./navigation-menu.mjs";
import { NumberField } from "./number-field.mjs";
import { PageHeader, PageViewport, pageHeaderClass, pageHeaderDescriptionClass, pageHeaderTitleClass, pageViewportClass, pageViewportContentClass } from "./page.mjs";
import { Onboarding, OnboardingDescription, OnboardingFooter, OnboardingHeader, OnboardingHeading, OnboardingTitle } from "./onboarding.mjs";
import { Progress, ProgressCircle, ProgressFill, ProgressLabel, ProgressRoot, ProgressTrack, ProgressValueLabel, normalizeProgressValue } from "./progress.mjs";
import { PromptComposer, PromptComposerAction, PromptComposerEditor, PromptComposerStatus, PromptComposerToolbar, PromptComposerTools, promptComposerClass, promptComposerEditorHeightClass } from "./prompt-composer.mjs";
import { PromptSuggestion, PromptSuggestions } from "./prompt-suggestion.mjs";
import { PropertyList, PropertyRow } from "./property-list.mjs";
import { QRCode, encodeQrCode, qrCodePath } from "./qr-code.mjs";
import { clampRatingValue, normalizeRatingMax, ratingLabel } from "./rating-state.mjs";
import { Rating } from "./rating.mjs";
import { Reasoning, ReasoningContent, ReasoningTrigger } from "./reasoning.mjs";
import { ResizableHandle, ResizablePanel, ResizablePanelGroup, createResizablePanelState, validateResizableSizes } from "./resizable.mjs";
import { SearchField } from "./search-field.mjs";
import { Select } from "./select.mjs";
import { Checkbox, RadioGroup, RadioGroupItem, Toggle, ToggleGroup, ToggleGroupItem } from "./selection.mjs";
import { SettingsGroup, SettingsItem, SettingsSection } from "./settings.mjs";
import { ShaderLayer } from "./shader-layer.mjs";
import { Sheet } from "./sheet.mjs";
import { ShortcutRecorder, shortcutFromKeyEvent } from "./shortcut-recorder.mjs";
import { workbenchClass, workbenchContentClass, workbenchContentColumnClass, workbenchFooterClass, workbenchHeaderClass, workbenchInspectorClass, workbenchInspectorContentClass, workbenchInspectorHeaderClass, workbenchMainClass, workbenchSidebarClass } from "./workbench-style.mjs";
import { Sidebar, SidebarContent, SidebarEmpty, SidebarFooter, SidebarGroup, SidebarGroupLabel, SidebarHeader, SidebarMenu, SidebarMenuButton, SidebarMenuIcon, SidebarMenuLabel, SidebarMenuSuffix, SidebarSearch, filterSidebarGroups } from "./sidebar.mjs";
import { Slider } from "./slider.mjs";
import { SplitButton } from "./split-button.mjs";
import { StatCard } from "./stat-card.mjs";
import { StatusBar, StatusBarGroup, StatusBarIndicator, StatusBarItem, StatusBarSeparator } from "./status-bar.mjs";
import { Stepper } from "./stepper.mjs";
import { Switch } from "./switch.mjs";
import { Tabs, TabsContent, TabsItem, TabsList, TabsTrigger } from "./tabs.mjs";
import { Timeline } from "./timeline.mjs";
import { TitleBar, TitleBarDragRegion, WindowFrame, titleBarClass, titleBarDragRegionLayoutStyle, titleBarLayoutStyle, windowFrameBackdropClassList, windowFrameClientClassList, windowFrameMaterialStyle, windowFrameShadows } from "./title-bar.mjs";
import { Toaster, createToasts } from "./toast.mjs";
import { Tool, ToolCodeSection, ToolContent, ToolHeader, ToolInput, ToolOutput, toolHeaderLabel } from "./tool.mjs";
import { TreeView, createTreeModel } from "./tree-view.mjs";
import { TypographyBlockquote, TypographyH1, TypographyH2, TypographyH3, TypographyH4, TypographyInlineCode, TypographyLarge, TypographyLead, TypographyList, TypographyListItem, TypographyMuted, TypographyP, TypographySmall } from "./typography.mjs";
import { Workbench, WorkbenchContent, WorkbenchContentColumn, WorkbenchFooter, WorkbenchHeader, WorkbenchInspector, WorkbenchInspectorContent, WorkbenchInspectorHeader, WorkbenchInspectorState, WorkbenchInspectorTitlebar, WorkbenchMain, WorkbenchSidebar } from "./workbench.mjs";
import { mergeClasses } from "@wabou/core/style";
import { createComponent, createFps, memo } from "@wabou/core/renderer";
import { P, match } from "ts-pattern";
//#region src/components/index.tsx
/** Frame-rate indicator. Live measurement is intentionally opt-in because it
* keeps the platform frame clock active. */
function Fps(props) {
	const measured = props.live ? createFps() : () => props.value;
	const value = () => Math.max(0, Math.round(measured()));
	const variant = () => match(value()).with(0, () => "outline").with(P.when((fps) => fps >= (props.goodAt ?? 55)), () => "success").with(P.when((fps) => fps < (props.warningBelow ?? 30)), () => "destructive").otherwise(() => "secondary");
	return createComponent(Badge, {
		get variant() {
			return variant();
		},
		weight: "normal",
		get "class"() {
			return mergeClasses("font-mono", props.class);
		},
		get children() {
			return [memo(() => {
				return value();
			}), memo(() => {
				return memo(() => {
					return props.label === "";
				})() ? "" : ` ${props.label ?? "fps"}`;
			})];
		}
	});
}
//#endregion
export { Accordion, AccordionContent, AccordionItem, AccordionTrigger, ActivityStatus, ActivityStatusIndicator, AdaptiveSplitPane, AdaptiveSplitPaneDetail, AdaptiveSplitPaneMain, Alert, AlertActions, AlertDescription, AlertDialog, AlertDialogAction, AlertDialogCancel, AlertDialogDescription, AlertDialogFooter, AlertDialogHeader, AlertDialogTitle, AlertTitle, AnnotationLayer, AspectRatio, Attachment, AttachmentAction, AttachmentActions, AttachmentContent, AttachmentDescription, AttachmentGroup, AttachmentMedia, AttachmentTitle, Avatar, AvatarGroup, AvatarGroupCount, Badge, Breadcrumb, BreadcrumbEllipsis, BreadcrumbItem, BreadcrumbLink, BreadcrumbList, BreadcrumbPage, BreadcrumbSeparator, Bubble, BubbleContent, BubbleGroup, BubbleReactions, Button, ButtonGroup, ButtonGroupSeparator, ButtonGroupText, Calendar, CalendarDate, Card, CardAction, CardContent, CardDescription, CardFooter, CardHeader, CardTitle, Carousel, CarouselContent, CarouselItem, CarouselNext, CarouselPrevious, ChartContainer, ChartEmpty, ChartLegend, Checkbox, CodeBlock, Collapsible, CollapsibleContent, CollapsibleTrigger, Combobox, Command, CommandList, ComponentsProvider, ContentState, ContextMenu, CopyButton, DataTable, DatePicker, DevServerErrorOverlay, Dialog, DialogDescription, DialogFooter, DialogHeader, DialogScrollBody, DialogTitle, DiffViewer, DirectionProvider, DirectionalRow, DirectionalText, DirectoryPicker, Drawer, DrawerClose, DrawerDescription, DrawerFooter, DrawerHandle, DrawerHeader, DrawerTitle, DropZone, DropdownMenu, Empty, EmptyContent, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle, Field, FieldContent, FieldDescription, FieldError, FieldGroup, FieldLabel, FieldLegend, FieldSeparator, FieldSet, FieldTitle, Fps, GroupBox, HoverCard, IconFrame, ImageList, ImageOverlayLayer, ImageViewport, InlineEdit, Input, InputGroup, InputGroupAddon, InputGroupButton, InputGroupInput, InputGroupText, InputGroupTextArea, InputOTP, InputOTPGroup, InputOTPSeparator, InputOTPSlot, Item, ItemActions, ItemContent, ItemDescription, ItemFooter, ItemGroup, ItemHeader, ItemMedia, ItemSeparator, ItemTitle, Kbd, KbdGroup, Label, LabeledField, LabeledSeparator, Listbox, Markdown, Marker, MarkerContent, MarkerIcon, Menubar, MenubarMenu, Message, MessageActions, MessageAvatar, MessageContent, MessageFooter, MessageGroup, MessageHeader, MessageScroller, MessageScrollerButton, MessageScrollerContent, MessageScrollerItem, MessageScrollerNavigator, MessageScrollerViewport, NavigationMenu, NavigationMenuContent, NavigationMenuIndicator, NavigationMenuItem, NavigationMenuLink, NavigationMenuList, NavigationMenuTrigger, NavigationMenuViewport, NumberField, Onboarding, OnboardingDescription, OnboardingFooter, OnboardingHeader, OnboardingHeading, OnboardingTitle, PageHeader, PageViewport, Pagination, PaginationContent, PaginationEllipsis, PaginationItem, PaginationItems, PaginationLink, PaginationNext, PaginationPrevious, PasswordInput, Popover, PopoverDescription, PopoverFooter, PopoverHeader, PopoverTitle, Progress, ProgressCircle, ProgressFill, ProgressLabel, ProgressRoot, ProgressTrack, ProgressValueLabel, PromptComposer, PromptComposerAction, PromptComposerEditor, PromptComposerStatus, PromptComposerToolbar, PromptComposerTools, PromptSuggestion, PromptSuggestions, PropertyList, PropertyRow, QRCode, RadioGroup, RadioGroupItem, Rating, Reasoning, ReasoningContent, ReasoningTrigger, ResizableHandle, ResizablePanel, ResizablePanelGroup, ResourceBoundary, ResponsiveGrid, ResponsiveGridRemainder, SearchField, Select, Separator, SettingsGroup, SettingsItem, SettingsSection, ShaderLayer, Sheet, DialogDescription as SheetDescription, DialogFooter as SheetFooter, DialogHeader as SheetHeader, DialogScrollBody as SheetScrollBody, DialogTitle as SheetTitle, ShortcutRecorder, Sidebar, SidebarContent, SidebarEmpty, SidebarFooter, SidebarGroup, SidebarGroupLabel, SidebarHeader, SidebarMenu, SidebarMenuButton, SidebarMenuIcon, SidebarMenuLabel, SidebarMenuSuffix, SidebarSearch, Skeleton, Slider, Spinner, SplitButton, SplitPane, SplitPaneAside, SplitPaneMain, StatCard, StatusBar, StatusBarGroup, StatusBarIndicator, StatusBarItem, StatusBarSeparator, Stepper, Switch, Table, TableBody, TableCaption, TableCell, TableFooter, TableHead, TableHeader, TableRow, TableSortIndicator, Tabs, TabsContent, TabsItem, TabsList, TabsTrigger, TextArea, Timeline, TitleBar, TitleBarDragRegion, Toaster, Toggle, ToggleGroup, ToggleGroupItem, Tool, ToolCodeSection, ToolContent, ToolHeader, ToolInput, ToolOutput, Toolbar, ToolbarButton, ToolbarGroup, ToolbarSeparator, ToolbarToggle, Tooltip, TooltipContent, TreeView, TypographyBlockquote, TypographyH1, TypographyH2, TypographyH3, TypographyH4, TypographyInlineCode, TypographyLarge, TypographyLead, TypographyList, TypographyListItem, TypographyMuted, TypographyP, TypographySmall, WindowFrame, Workbench, WorkbenchContent, WorkbenchContentColumn, WorkbenchFooter, WorkbenchHeader, WorkbenchInspector, WorkbenchInspectorContent, WorkbenchInspectorHeader, WorkbenchInspectorState, WorkbenchInspectorTitlebar, WorkbenchMain, WorkbenchSidebar, activeMessageAnchor, alertColors, aspectRatioStyle, attachmentClass, attachmentGroupClass, attachmentMediaClass, avatarInitials, badgeClass, bubbleClass, bubbleContentClass, clampAnnotationRegion, clampPage, clampRatingValue, componentToneClass, componentsControlSize, componentsElevation, componentsThemeContract, createCommandListNavigation, createDelayedOpenController, createPaginationRange, createResizablePanelState, createToasts, createDelayedOpenController as createTooltipDelayController, createTreeModel, drawerDragOffset, drawerShouldDismiss, emptyClass, emptyMediaClass, encodeQrCode, fieldClass, fieldErrorLabel, filterCommandItems, filterSidebarGroups, groupBoxContentClass, imageViewportTransform, inputGroupAddonClass, inputGroupClass, isMessageScrollNearEnd, itemClass, itemMediaClass, messageActionsClass, messageClass, messageContentClass, messageScrollRange, messageScrollRevealDelta, messageScrollStartDelta, moveMenuHighlight, navigationMenuTriggerClass, nextAccordionValue, normalizeCarouselIndex, normalizeOtpValue, normalizePageCount, normalizeProgressValue, normalizeRatingMax, pageHeaderClass, pageHeaderDescriptionClass, pageHeaderTitleClass, pageViewportClass, pageViewportContentClass, pointInLayoutRect, promptComposerClass, promptComposerEditorHeightClass, qrCodePath, ratingLabel, reconcileCommandHighlight, responsiveGridColumnCount, responsiveGridRemainderCount, shortcutFromKeyEvent, titleBarClass, titleBarDragRegionLayoutStyle, titleBarLayoutStyle, toolHeaderLabel, uniqueFieldErrors, useChartConfig, useComponentsTheme, useDirection, useMessageScroller, useResponsiveGrid, validateResizableSizes, windowFrameBackdropClassList, windowFrameClientClassList, windowFrameMaterialStyle, windowFrameShadows, workbenchClass, workbenchContentClass, workbenchContentColumnClass, workbenchFooterClass, workbenchHeaderClass, workbenchInspectorClass, workbenchInspectorContentClass, workbenchInspectorHeaderClass, workbenchMainClass, workbenchSidebarClass };

//# sourceMappingURL=index.mjs.map