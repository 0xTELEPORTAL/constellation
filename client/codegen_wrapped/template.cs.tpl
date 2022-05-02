// This file is autogenerated, with one per class.

using IntPtr = System.IntPtr;
using generated = Teleportal.Client.Generated.generated;


namespace Teleportal.Client.Contract.Properties
{
    partial class ToManaged
    {
        public static unsafe Channels.{{class_ident}} f(OwnershipSemantics ownershipSemantics, Ptr<Channels.{{class_ident}}> ptr)
        {
            return new Channels.{{class_ident}}(ptr, ownershipSemantics);
        }
    }
}

namespace Teleportal.Client.Contract.Properties.Channels
{
    public sealed class {{class_ident}} : Wrapper<{{class_ident}}>
    {
        public {{class_ident}}(Ptr<{{class_ident}}> ptr, OwnershipSemantics ownershipSemantics) : base(ptr, ownershipSemantics) { }

        {{#if new_expr}}
        public unsafe {{class_ident}}({{new_args}}) : base(
            new Ptr<{{class_ident}}>({{new_expr}}),
            OwnershipSemantics.Owned
        )
        { }
        {{/if}}

        override protected void NativeDrop(Ptr<{{class_ident}}> ptr)
        {
            {{drop_ident}}(ptr.p);
        }

        {{> additional_methods}}
    }
}