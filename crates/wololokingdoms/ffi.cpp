#include <map>
#include <wksettings.h>
#include <wkconverter.h>

typedef WKConverter* wkconverter_t;
typedef WKSettings* wksettings_t;

struct wklistener_callbacks {
  void* data;
  void (*finished) (void*);
  void (*log) (void*, const char*);
  void (*set_info) (void*, const char*);
  void (*error) (void*, const char*);
  void (*create_dialog) (void*, const char*);
  void (*create_dialog_title) (void*, const char*, const char*);
  void (*create_dialog_replace) (void*, const char*, const char*, const char*);
  void (*set_progress) (void*, int);
  void (*install_userpatch) (void*, const char*, const char**);
};
typedef struct wklistener_callbacks* wklistener_t;

class FFIConvertListener: public WKConvertListener {
  wklistener_t listener;
public:
  FFIConvertListener (wklistener_t listener) : listener(listener) {}
  virtual void finished () {
    if (listener->finished) {
      listener->finished(listener->data);
    }
  }
  virtual void log(std::string msg) {
    if (listener->log) {
      listener->log(listener->data, msg.c_str());
    }
  }
  virtual void setInfo(std::string msg) {
    if (listener->set_info) {
      listener->set_info(listener->data, msg.c_str());
    }
  }
  virtual void error(std::exception const& err) {
    if (listener->error) {
      printf("FFIConvertListener#error(%p, %s)\n", listener->data, err.what());
      listener->error(listener->data, err.what());
    }
  }
  virtual void setProgress(int i) {
    if (listener->set_progress) {
      listener->set_progress(listener->data, i);
    }
  }
  virtual void installUserPatch(fs::path userPatchExe, std::vector<std::string> cliFlags) {
    if (!listener->install_userpatch) {
      return;
    }

    auto num_flags = cliFlags.size();
    auto flags = new const char*[num_flags + 1];
    auto i = 0;
    for (auto& f : cliFlags) {
      flags[i++] = f.c_str();
    }
    listener->install_userpatch(listener->data, userPatchExe.string().c_str(), flags);
  }
};

extern "C" wksettings_t wksettings_create () {
  return new WKSettings();
}

extern "C" void wksettings_use_voobly (wksettings_t settings, bool use_voobly) {
  settings->useVoobly = use_voobly;
}
extern "C" void wksettings_use_exe (wksettings_t settings, bool use_exe) {
  settings->useExe = use_exe;
}
extern "C" void wksettings_use_both (wksettings_t settings, bool use_both) {
  settings->useBoth = use_both;
}
extern "C" void wksettings_use_monks (wksettings_t settings, bool use_monks) {
  settings->useMonks = use_monks;
}
extern "C" void wksettings_use_small_trees (wksettings_t settings, bool use_small_trees) {
  settings->useSmallTrees = use_small_trees;
}
extern "C" void wksettings_use_short_walls (wksettings_t settings, bool use_short_walls) {
  settings->useShortWalls = use_short_walls;
}
extern "C" void wksettings_copy_maps (wksettings_t settings, bool copy_maps) {
  settings->copyMaps = copy_maps;
}
extern "C" void wksettings_copy_custom_maps (wksettings_t settings, bool copy_custom_maps) {
  settings->copyCustomMaps = copy_custom_maps;
}
extern "C" void wksettings_restricted_civ_mods (wksettings_t settings, bool restricted_civ_mods) {
  settings->restrictedCivMods = restricted_civ_mods;
}
extern "C" void wksettings_use_no_snow (wksettings_t settings, bool use_no_snow) {
  settings->useNoSnow = use_no_snow;
}
extern "C" void wksettings_use_grid (wksettings_t settings, bool use_grid) {
  settings->useGrid = use_grid;
}
extern "C" void wksettings_fix_flags (wksettings_t settings, bool fix_flags) {
  settings->fixFlags = fix_flags;
}
extern "C" void wksettings_replace_tooltips (wksettings_t settings, bool replace_tooltips) {
  settings->replaceTooltips = replace_tooltips;
}
extern "C" void wksettings_hd_path (wksettings_t settings, char* path) {
  settings->hdPath = path;
}
extern "C" void wksettings_out_path (wksettings_t settings, char* path) {
  settings->outPath = path;
}
extern "C" void wksettings_voobly_path (wksettings_t settings, char* path) {
  settings->vooblyDir = path;
}
extern "C" void wksettings_up_path (wksettings_t settings, char* path) {
  settings->upDir = path;
}

extern "C" void wksettings_destroy (wksettings_t settings) {
  delete settings;
}

extern "C" wkconverter_t wkconverter_create (wksettings_t settings, wklistener_t listener) {
  printf("wkconverter_create(%p, %p)\n", settings, listener);
  auto convert_listener = new FFIConvertListener(listener);
  auto converter = new WKConverter(settings, convert_listener);
  return converter;
}

extern "C" void wkconverter_run (wkconverter_t converter) {
  printf("wkconverter_run(%p)\n", converter);
  converter->run();
}

extern "C" void wkconverter_destroy (wkconverter_t converter) {
  delete converter;
}
