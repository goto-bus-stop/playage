typedef void* wkconverter_t;
typedef void* wksettings_t;

struct wklistener_callbacks {
  void* data;
  void (*finished) (const void*);
  void (*log) (const void*, const char*);
  void (*set_info) (const void*, const char*);
  void (*error) (const void*, const char*);
  void (*create_dialog) (const void*, const char*);
  void (*create_dialog_title) (const void*, const char*, const char*);
  void (*create_dialog_replace) (const void*, const char*, const char*, const char*);
  void (*set_progress) (const void*, int);
  void (*install_userpatch) (const void*, const char*, const char* const*);
};
typedef struct wklistener_callbacks* wklistener_t;

wksettings_t wksettings_create ();
void wksettings_use_voobly (wksettings_t settings, char use_voobly);
void wksettings_use_exe (wksettings_t settings, char use_exe);
void wksettings_use_both (wksettings_t settings, char use_both);
void wksettings_use_monks (wksettings_t settings, char use_monks);
void wksettings_copy_maps (wksettings_t settings, char copy_maps);
void wksettings_copy_custom_maps (wksettings_t settings, char copy_custom_maps);
void wksettings_restricted_civ_mods (wksettings_t settings, char restricted_civ_mods);
void wksettings_use_grid (wksettings_t settings, char use_grid);
void wksettings_fix_flags (wksettings_t settings, char fix_flags);
void wksettings_replace_tooltips (wksettings_t settings, char replace_tooltips);
void wksettings_hd_path (wksettings_t settings, const char* path);
void wksettings_out_path (wksettings_t settings, const char* path);
void wksettings_voobly_path (wksettings_t settings, const char* path);
void wksettings_up_path (wksettings_t settings, const char* path);

void wksettings_destroy (wksettings_t settings);

wkconverter_t wkconverter_create (wksettings_t settings, wklistener_t listener);

void wkconverter_run (wkconverter_t converter);

void wkconverter_destroy (wkconverter_t converter);
