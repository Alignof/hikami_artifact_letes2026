/* trace_insn.c
 * gcc -shared -fPIC \
 -I/nix/store/xn6y3xxy1zm8ykf3lf9f5xdx29kjxmgf-qemu-10.1.2/include \
    $(pkg-config --cflags --libs glib-2.0) \
    trace_pc.c -o libtrace_pc.so
 */

#include <glib.h>
#include <qemu-plugin.h>
#include <stdio.h>

QEMU_PLUGIN_EXPORT int qemu_plugin_version = QEMU_PLUGIN_VERSION;

static uint64_t start_pc = 0;
static uint64_t end_pc = 0;
static uint64_t trigger_pc = 0;
static uint64_t stop_pc = 0;

static bool tracing_active = true;
static bool trigger_enabled = false;

typedef struct {
    uint64_t pc;
    char* disas;
    uint32_t insn_data;
    int insn_len;
} InsnInfo;

static void vcpu_insn_exec(unsigned int cpu_index, void* userdata) {
    InsnInfo* info = (InsnInfo*)userdata;
    uint64_t pc = info->pc;

    if (!tracing_active) {
        if (trigger_enabled && pc == trigger_pc) {
            tracing_active = true;
            qemu_plugin_outs(
                "--- Trigger address reached. Tracing started. ---\n");
        } else {
            return;
        }
    }

    if (stop_pc != 0 && pc == stop_pc) {
        tracing_active = false;
        qemu_plugin_outs("--- Stop address reached. Tracing finished. ---\n");
        return;
    }

    bool in_log_range =
        (start_pc == 0 && end_pc == 0) || (pc >= start_pc && pc <= end_pc);

    if (in_log_range || pc == trigger_pc || pc == stop_pc) {
        g_autofree char* msg = NULL;
        if (info->insn_len == 4) {
            msg = g_strdup_printf("0x%lx: [%08x] %s\n", pc, info->insn_data,
                                  info->disas);
        } else {
            msg = g_strdup_printf("0x%lx: [    %04x] %s\n", pc,
                                  info->insn_data & 0xFFFF, info->disas);
        }
        qemu_plugin_outs(msg);
    }
}

static void vcpu_tb_trans(qemu_plugin_id_t id, struct qemu_plugin_tb* tb) {
    size_t n = qemu_plugin_tb_n_insns(tb);
    for (size_t i = 0; i < n; i++) {
        struct qemu_plugin_insn* insn = qemu_plugin_tb_get_insn(tb, i);
        uint64_t vaddr = qemu_plugin_insn_vaddr(insn);

        bool in_range = (start_pc == 0 && end_pc == 0) ||
                        (vaddr >= start_pc && vaddr <= end_pc);

        bool is_control_point = (vaddr == trigger_pc) || (vaddr == stop_pc);

        if (in_range || is_control_point) {
            InsnInfo* info = g_new0(InsnInfo, 1);
            info->pc = vaddr;
            info->disas = qemu_plugin_insn_disas(insn);
            info->insn_len = qemu_plugin_insn_size(insn);

            uint32_t data = 0;
            qemu_plugin_insn_data(insn, &data, info->insn_len);
            info->insn_data = data;

            qemu_plugin_register_vcpu_insn_exec_cb(
                insn, vcpu_insn_exec, QEMU_PLUGIN_CB_NO_REGS, info);
        }
    }
}

QEMU_PLUGIN_EXPORT int qemu_plugin_install(qemu_plugin_id_t id,
                                           const qemu_info_t* info, int argc,
                                           char** argv) {
    for (int i = 0; i < argc; i++) {
        char* opt = argv[i];
        g_autofree char** tokens = g_strsplit(opt, "=", 2);
        if (g_strcmp0(tokens[0], "start") == 0) {
            start_pc = g_ascii_strtoull(tokens[1], NULL, 16);
        } else if (g_strcmp0(tokens[0], "end") == 0) {
            end_pc = g_ascii_strtoull(tokens[1], NULL, 16);
        } else if (g_strcmp0(tokens[0], "trigger") == 0) {  // 追加
            trigger_pc = g_ascii_strtoull(tokens[1], NULL, 16);
            trigger_enabled = true;
            tracing_active = false;  // トリガー有効なら最初は停止
        } else if (g_strcmp0(tokens[0], "stop") == 0) {
            stop_pc = g_ascii_strtoull(tokens[1], NULL, 16);
        }
    }

    qemu_plugin_register_vcpu_tb_trans_cb(id, vcpu_tb_trans);
    return 0;
}
