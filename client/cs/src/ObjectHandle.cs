using generated = tp_client.generated;
using RSharp;
using System.Collections.Generic;

// This file is manually implemented for now but will be autogenerated eventually

namespace Teleportal.Client.Object
{
    public sealed class ObjectHandle : OpaqueWrapper<ObjectHandle>
    {
        public ObjectHandle(Ptr<ObjectHandle> inner) : base(inner, OwnershipSemantics.Owned) { }

        override protected void NativeDrop(Ptr<ObjectHandle> inner)
        {
            generated.__Internal.TpClientObjectObjectHandleDrop(inner.p);
        }
    }

}