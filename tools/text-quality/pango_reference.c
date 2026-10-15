#include <cairo.h>
#include <pango/pangocairo.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static void parse_color(const char *value, double *r, double *g, double *b) {
    unsigned int color = 0;
    if (value[0] == '#') value++;
    if (sscanf(value, "%06x", &color) != 1) {
        fprintf(stderr, "invalid color: %s\n", value);
        exit(2);
    }
    *r = ((color >> 16) & 0xff) / 255.0;
    *g = ((color >> 8) & 0xff) / 255.0;
    *b = (color & 0xff) / 255.0;
}

static cairo_hint_style_t hint_style(const char *value) {
    if (strcmp(value, "none") == 0) return CAIRO_HINT_STYLE_NONE;
    if (strcmp(value, "slight") == 0) return CAIRO_HINT_STYLE_SLIGHT;
    if (strcmp(value, "medium") == 0) return CAIRO_HINT_STYLE_MEDIUM;
    if (strcmp(value, "full") == 0) return CAIRO_HINT_STYLE_FULL;
    return CAIRO_HINT_STYLE_DEFAULT;
}

static cairo_antialias_t antialias(const char *value) {
    if (strcmp(value, "none") == 0) return CAIRO_ANTIALIAS_NONE;
    if (strcmp(value, "gray") == 0) return CAIRO_ANTIALIAS_GRAY;
    if (strcmp(value, "subpixel") == 0) return CAIRO_ANTIALIAS_SUBPIXEL;
    return CAIRO_ANTIALIAS_DEFAULT;
}

static PangoWeight pango_weight(int value) {
    if (value <= 150) return PANGO_WEIGHT_THIN;
    if (value <= 250) return PANGO_WEIGHT_ULTRALIGHT;
    if (value <= 350) return PANGO_WEIGHT_LIGHT;
    if (value <= 450) return PANGO_WEIGHT_NORMAL;
    if (value <= 550) return PANGO_WEIGHT_MEDIUM;
    if (value <= 650) return PANGO_WEIGHT_SEMIBOLD;
    if (value <= 750) return PANGO_WEIGHT_BOLD;
    if (value <= 850) return PANGO_WEIGHT_ULTRABOLD;
    return PANGO_WEIGHT_HEAVY;
}

int main(int argc, char **argv) {
    if (argc < 17 || (argc - 12) % 5 != 0) {
        fprintf(stderr,
                "usage: %s OUTPUT REPORT WIDTH HEIGHT SCALE FAMILY BG FG "
                "HINT ANTIALIAS SUBPIXEL X BASELINE SIZE WEIGHT TEXT ...\n",
                argv[0]);
        return 2;
    }

    const char *output = argv[1];
    const char *report_path = argv[2];
    int width = atoi(argv[3]);
    int height = atoi(argv[4]);
    double scale = atof(argv[5]);
    const char *family = argv[6];
    const char *background = argv[7];
    const char *foreground = argv[8];
    const char *hint = argv[9];
    const char *aa = argv[10];
    int subpixel_positions = atoi(argv[11]);

    cairo_surface_t *surface = cairo_image_surface_create(
        CAIRO_FORMAT_ARGB32, (int)(width * scale), (int)(height * scale));
    cairo_t *cr = cairo_create(surface);
    cairo_scale(cr, scale, scale);

    cairo_font_options_t *options = cairo_font_options_create();
    cairo_font_options_set_hint_style(options, hint_style(hint));
    cairo_font_options_set_hint_metrics(options, CAIRO_HINT_METRICS_ON);
    cairo_font_options_set_antialias(options, antialias(aa));
    cairo_font_options_set_subpixel_order(options, CAIRO_SUBPIXEL_ORDER_RGB);
    cairo_set_font_options(cr, options);

    double r, g, b;
    parse_color(background, &r, &g, &b);
    cairo_set_source_rgb(cr, r, g, b);
    cairo_paint(cr);
    parse_color(foreground, &r, &g, &b);
    cairo_set_source_rgb(cr, r, g, b);

    PangoLayout *layout = pango_cairo_create_layout(cr);
    PangoContext *context = pango_layout_get_context(layout);
    pango_cairo_context_set_font_options(context, options);
    pango_context_set_round_glyph_positions(context, !subpixel_positions);
    FILE *report = fopen(report_path, "w");
    if (!report) {
        perror("open report");
        return 2;
    }
    fprintf(report, "backend=pango-cairo\nscale=%.3f\nhint=%s\nantialias=%s\nsubpixel_positions=%d\n",
            scale, hint, aa, subpixel_positions);

    int row = 0;
    for (int i = 12; i < argc; i += 5, row++) {
        double x = atof(argv[i]);
        double baseline = atof(argv[i + 1]);
        double size = atof(argv[i + 2]);
        int weight = atoi(argv[i + 3]);
        const char *text = argv[i + 4];

        PangoFontDescription *description = pango_font_description_new();
        pango_font_description_set_family(description, family);
        pango_font_description_set_absolute_size(description, size * PANGO_SCALE);
        pango_font_description_set_weight(description, pango_weight(weight));
        pango_layout_set_font_description(layout, description);
        pango_layout_set_text(layout, text, -1);
        pango_layout_context_changed(layout);

        double layout_baseline = pango_layout_get_baseline(layout) / (double)PANGO_SCALE;
        cairo_move_to(cr, x, baseline - layout_baseline);
        pango_cairo_show_layout(cr, layout);

        PangoRectangle logical;
        pango_layout_get_pixel_extents(layout, NULL, &logical);
        fprintf(report,
                "row=%d size=%.3f weight=%d baseline=%.3f logical=%d,%d,%d,%d text=%s\n",
                row, size, weight, layout_baseline, logical.x, logical.y,
                logical.width, logical.height, text);
        PangoLayoutIter *iter = pango_layout_get_iter(layout);
        do {
            PangoLayoutRun *run = pango_layout_iter_get_run_readonly(iter);
            if (run && run->item && run->item->analysis.font) {
                PangoFontDescription *actual =
                    pango_font_describe(run->item->analysis.font);
                char *actual_name = pango_font_description_to_string(actual);
                fprintf(report, "  font=%s glyphs=", actual_name);
                for (int glyph = 0; glyph < run->glyphs->num_glyphs; glyph++) {
                    if (glyph) fputc(',', report);
                    fprintf(report, "%u:%d", run->glyphs->glyphs[glyph].glyph,
                            run->glyphs->glyphs[glyph].geometry.width);
                }
                fputc('\n', report);
                g_free(actual_name);
                pango_font_description_free(actual);
            }
        } while (pango_layout_iter_next_run(iter));
        pango_layout_iter_free(iter);
        pango_font_description_free(description);
    }

    fclose(report);
    cairo_status_t status = cairo_surface_write_to_png(surface, output);
    g_object_unref(layout);
    cairo_font_options_destroy(options);
    cairo_destroy(cr);
    cairo_surface_destroy(surface);
    if (status != CAIRO_STATUS_SUCCESS) {
        fprintf(stderr, "write png: %s\n", cairo_status_to_string(status));
        return 1;
    }
    return 0;
}
