use crate::{generate_class_data_generic, ClassData, ClassDataTemplate};

use indoc::indoc;

pub struct KeyframeTemplate {}

impl ClassDataTemplate for KeyframeTemplate {
    fn namespace_super() -> String {
        "Contract.Properties".to_string()
    }

    fn namespace_sub() -> String {
        "Channels".to_string()
    }

    fn class_ident() -> String {
        "Keyframe_<type_platform>".to_string()
    }

    fn new_args() -> String {
        "<type_cs> value, double time".to_string()
    }

    fn new_expr() -> Option<String> {
        Some("generated.__Internal.TpClientContractPropertiesChannelsKeyframe<type_platform>New(RSharp.RBox_<type_platform>.new_(value), time)".to_string())
    }

    fn drop_ident() -> String {
        "generated.__Internal.TpClientContractPropertiesChannelsKeyframe<type_platform>Drop"
            .to_string()
    }

    fn additional_methods() -> Option<String> {
        Some(indoc! {r#"
            public unsafe <type_cs> Value
            {
                get
                {
                    var result = generated.__Internal.TpClientContractPropertiesChannelsKeyframe<type_platform>Value(this.Ptr?.p ?? IntPtr.Zero);
                    return ToManaged.f(OwnershipSemantics.SharedRef, result);
                }
            }

            public double Time
            {
                get => generated.__Internal.TpClientContractPropertiesChannelsKeyframe<type_platform>Time(this.Ptr?.p ?? IntPtr.Zero);
            }
        "#}.to_string())
    }

    fn generate_class_data() -> Vec<ClassData> {
        generate_class_data_generic::<Self>()
    }
}
