use gdnative::api::Button;
use gdnative::prelude::*;
use crate::prodot_builder::*;

#[derive(NativeClass)]
#[inherit(Button)]
#[register_with(Self::register_signals)]
pub struct FaceModeButton; 

#[methods]
impl FaceModeButton {
    fn new(_owner: TRef<Button>) -> Self {
        FaceModeButton
    }

    fn register_signals(builder: &ClassBuilder<Self>) {
        builder.add_signal(Signal {
            name: "face_mode",
            args: &[SignalArgument {
                name: "mode",
                default: Variant::from_i64(BuildMode::Vertex.value()),
                export_info: ExportInfo::new(VariantType::I64),
                usage: PropertyUsage::DEFAULT,
            }],
        });
    }

    #[export]
    fn _enter_tree(&self, owner: TRef<Button>) {
        owner
            .connect("pressed", owner, "on_click", VariantArray::new_shared(), 0)
            .unwrap();
    }

    #[export]
    fn on_click(&self, owner: TRef<Button>) {
        owner.emit_signal("face_mode", &[ Variant::from_i64(BuildMode::Face.value()) ] );
    }
}
