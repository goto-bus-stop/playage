#include <map>
#include <wksettings.h>
#include <wkconverter.h>

typedef void* wkconverter_t;

struct wksettings_t {
  bool use_voobly;
  bool use_exe;
  bool use_both;
  bool use_regional_monks;
  bool use_small_trees;
  bool use_short_walls;
  bool copy_maps;
  bool copy_custom_maps;
  bool restricted_civ_mods;
  bool use_no_snow;
  bool fix_flags;
  bool replace_tooltips;
  bool use_grid;
  char* language;
  int dlc_level;
  int patch;
  int hotkey_choice;
  char* hd_directory;
  char* aoc_directory;
  char* voobly_directory;
  char* userpatch_directory;
  char* mod_name;
};

struct wklistener_t {
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

class FFIConvertListener: public WKConvertListener {
  wklistener_t* listener;
public:
  FFIConvertListener (wklistener_t* listener) : listener(listener) {}
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

extern "C" wkconverter_t wkconverter_create (wksettings_t* settings, wklistener_t* listener) {
  printf("wkconverter_create(%p, %p)\n", settings, listener);

  printf("use_voobly %d\n", settings->use_voobly);
  printf("use_exe %d\n", settings->use_exe);
  printf("use_both %d\n", settings->use_both);
  printf("use_regional_monks %d\n", settings->use_regional_monks);
  printf("use_small_trees %d\n", settings->use_small_trees);
  printf("use_short_walls %d\n", settings->use_short_walls);
  printf("copy_maps %d\n", settings->copy_maps);
  printf("copy_custom_maps %d\n", settings->copy_custom_maps);
  printf("restricted_civ_mods %d\n", settings->restricted_civ_mods);
  printf("use_no_snow %d\n", settings->use_no_snow);
  printf("fix_flags %d\n", settings->fix_flags);
  printf("replace_tooltips %d\n", settings->replace_tooltips);
  printf("use_grid %d\n", settings->use_grid);
  printf("language %s\n", settings->language);
  printf("dlc_level %d\n", settings->dlc_level);
  printf("patch %d\n", settings->patch);
  printf("hotkey_choice %d\n", settings->hotkey_choice);
  printf("hd_directory %s\n", settings->hd_directory);
  printf("aoc_directory %s\n", settings->aoc_directory);
  printf("voobly_directory %s\n", settings->voobly_directory);
  printf("userpatch_directory %s\n", settings->userpatch_directory);
  printf("mod_name %s\n", settings->mod_name);

  auto convert_settings = new WKSettings(
    settings->use_voobly,
    settings->use_exe,
    settings->use_both,
    settings->use_regional_monks,
    settings->use_small_trees,
    settings->use_short_walls,
    settings->copy_maps,
    settings->copy_custom_maps,
    settings->restricted_civ_mods,
    settings->use_no_snow,
    settings->fix_flags,
    settings->replace_tooltips,
    settings->use_grid,
    "", // not used
    settings->language,
    settings->dlc_level,
    settings->patch,
    settings->hotkey_choice,
    settings->hd_directory,
    settings->aoc_directory,
    settings->voobly_directory,
    settings->userpatch_directory,
    std::map<int, std::tuple<std::string,std::string, std::string, int, std::string>>(),
    settings->mod_name
  );
  auto convert_listener = new FFIConvertListener(listener);


  auto converter = new WKConverter(convert_settings, convert_listener);

  return converter;
}

extern "C" void wkconverter_run (wkconverter_t converter) {
  auto wkc = reinterpret_cast<WKConverter*>(converter);
  wkc->run();
}
